use std::{error::Error, fmt, path::PathBuf};

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    application::open_project::{open_project, OpenProjectError, OpenProjectInput},
    persistence::project_manifest::DATABASE_FILE,
};

const EDITOR_SCHEMA_VERSION: i64 = 1;
const MAX_RESULTS: usize = 200;
const SNIPPET_CONTEXT_CHARACTERS: usize = 40;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchProjectInput {
    pub project_path: PathBuf,
    pub query: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSearchMatch {
    pub document_id: String,
    pub document_title: String,
    pub snippet: String,
    pub from: usize,
    pub to: usize,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSearchResults {
    pub matches: Vec<ProjectSearchMatch>,
    pub truncated: bool,
}

#[derive(Debug)]
pub enum SearchProjectError {
    OpenProject(OpenProjectError),
    Database {
        path: PathBuf,
        source: rusqlite::Error,
    },
    UnsupportedContentSchema {
        document_id: String,
        schema_version: i64,
    },
    InvalidContent {
        document_id: String,
        details: String,
    },
}

impl SearchProjectError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::OpenProject(source) => source.code(),
            Self::Database { .. } => "project_search.failed",
            Self::UnsupportedContentSchema { .. } => "project_search.unsupported_content",
            Self::InvalidContent { .. } => "project_search.invalid_content",
        }
    }

    pub fn user_message(&self) -> &'static str {
        match self {
            Self::OpenProject(source) => source.user_message(),
            Self::Database { .. } => "Não foi possível buscar no projeto.",
            Self::UnsupportedContentSchema { .. } => {
                "Um documento usa uma versão de conteúdo ainda não suportada."
            }
            Self::InvalidContent { .. } => "Os dados de um documento estão inconsistentes.",
        }
    }
}

impl fmt::Display for SearchProjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenProject(source) => write!(formatter, "cannot open project: {source}"),
            Self::Database { path, source } => write!(
                formatter,
                "project search database operation failed at {}: {source}",
                path.display()
            ),
            Self::UnsupportedContentSchema {
                document_id,
                schema_version,
            } => write!(
                formatter,
                "document {document_id} uses unsupported content schema {schema_version}"
            ),
            Self::InvalidContent {
                document_id,
                details,
            } => write!(
                formatter,
                "document {document_id} contains invalid editor content: {details}"
            ),
        }
    }
}

impl Error for SearchProjectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OpenProject(source) => Some(source),
            Self::Database { source, .. } => Some(source),
            Self::UnsupportedContentSchema { .. } | Self::InvalidContent { .. } => None,
        }
    }
}

struct PersistedDocument {
    document_id: String,
    title: String,
    schema_version: i64,
    json_content: String,
}

struct SearchPattern {
    folded_query: String,
}

#[derive(Clone, Copy)]
struct TextBoundary {
    folded_byte: usize,
    original_byte: usize,
    original_utf16: usize,
}

pub fn search_project(
    input: SearchProjectInput,
) -> Result<ProjectSearchResults, SearchProjectError> {
    if input.query.is_empty() {
        return Ok(ProjectSearchResults {
            matches: Vec::new(),
            truncated: false,
        });
    }

    let project = open_project(OpenProjectInput {
        path: input.project_path,
    })
    .map_err(SearchProjectError::OpenProject)?;
    let database_path = project.path.join(DATABASE_FILE);
    let connection = Connection::open_with_flags(&database_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|source| database_error(&database_path, source))?;
    let documents = load_searchable_documents(&connection, &database_path)?;
    let pattern = SearchPattern::new(&input.query);
    let mut matches = Vec::new();

    for document in documents {
        if document.schema_version != EDITOR_SCHEMA_VERSION {
            return Err(SearchProjectError::UnsupportedContentSchema {
                document_id: document.document_id,
                schema_version: document.schema_version,
            });
        }

        let content: Value = serde_json::from_str(&document.json_content).map_err(|source| {
            SearchProjectError::InvalidContent {
                document_id: document.document_id.clone(),
                details: source.to_string(),
            }
        })?;
        search_document_content(&content, &document, &pattern, &mut matches).map_err(
            |details| SearchProjectError::InvalidContent {
                document_id: document.document_id.clone(),
                details,
            },
        )?;

        if matches.len() > MAX_RESULTS {
            break;
        }
    }

    let truncated = matches.len() > MAX_RESULTS;
    matches.truncate(MAX_RESULTS);
    Ok(ProjectSearchResults { matches, truncated })
}

fn load_searchable_documents(
    connection: &Connection,
    database_path: &std::path::Path,
) -> Result<Vec<PersistedDocument>, SearchProjectError> {
    let mut statement = connection
        .prepare(
            "SELECT documents.id, documents.title,
                    document_content.schema_version, document_content.json_content
             FROM documents
             JOIN document_content ON document_content.document_id = documents.id
             WHERE documents.kind = 'flow' AND documents.status <> 'trashed'
             ORDER BY documents.position, documents.id",
        )
        .map_err(|source| database_error(database_path, source))?;
    let documents = statement
        .query_map([], |row| {
            Ok(PersistedDocument {
                document_id: row.get(0)?,
                title: row.get(1)?,
                schema_version: row.get(2)?,
                json_content: row.get(3)?,
            })
        })
        .map_err(|source| database_error(database_path, source))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|source| database_error(database_path, source))?;
    Ok(documents)
}

fn search_document_content(
    content: &Value,
    document: &PersistedDocument,
    pattern: &SearchPattern,
    matches: &mut Vec<ProjectSearchMatch>,
) -> Result<(), String> {
    if node_type(content)? != "doc" {
        return Err("editor content root must be a doc node".to_owned());
    }

    let mut position = 0;
    for child in node_content(content)? {
        position += visit_node(child, position, document, pattern, matches)?;
    }
    Ok(())
}

fn visit_node(
    node: &Value,
    position: usize,
    document: &PersistedDocument,
    pattern: &SearchPattern,
    matches: &mut Vec<ProjectSearchMatch>,
) -> Result<usize, String> {
    let node_type = node_type(node)?;
    if node_type == "text" {
        return Err("text nodes must be inside a text block".to_owned());
    }
    if node_type == "paragraph" || node_type == "heading" {
        return search_text_block(node, position, document, pattern, matches);
    }

    let children = node_content(node)?;
    if children.is_empty() {
        return Ok(1);
    }

    let mut content_size = 0;
    for child in children {
        content_size += visit_node(
            child,
            position + 1 + content_size,
            document,
            pattern,
            matches,
        )?;
    }
    Ok(content_size + 2)
}

fn search_text_block(
    node: &Value,
    position: usize,
    document: &PersistedDocument,
    pattern: &SearchPattern,
    matches: &mut Vec<ProjectSearchMatch>,
) -> Result<usize, String> {
    let mut text = String::new();
    let mut content_size = 0;

    for child in node_content(node)? {
        match node_type(child)? {
            "text" => {
                let child_text = child
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "text node must contain text".to_owned())?;
                text.push_str(child_text);
                content_size += child_text.encode_utf16().count();
            }
            "hardBreak" => {
                text.push('\n');
                content_size += 1;
            }
            unsupported => {
                return Err(format!(
                    "unsupported inline node in text block: {unsupported}"
                ));
            }
        }
    }

    for found in pattern.find_in(&text) {
        matches.push(ProjectSearchMatch {
            document_id: document.document_id.clone(),
            document_title: document.title.clone(),
            snippet: create_snippet(&text, found.start_byte, found.end_byte),
            from: position + 1 + found.start_utf16,
            to: position + 1 + found.end_utf16,
        });
        if matches.len() > MAX_RESULTS {
            break;
        }
    }

    Ok(content_size + 2)
}

fn node_type(node: &Value) -> Result<&str, String> {
    node.as_object()
        .and_then(|object| object.get("type"))
        .and_then(Value::as_str)
        .ok_or_else(|| "editor node must contain a string type".to_owned())
}

fn node_content(node: &Value) -> Result<&[Value], String> {
    match node.get("content") {
        None => Ok(&[]),
        Some(Value::Array(content)) => Ok(content),
        Some(_) => Err("editor node content must be an array".to_owned()),
    }
}

struct FoundText {
    start_byte: usize,
    end_byte: usize,
    start_utf16: usize,
    end_utf16: usize,
}

impl SearchPattern {
    fn new(query: &str) -> Self {
        Self {
            folded_query: fold_case(query),
        }
    }

    fn find_in(&self, text: &str) -> Vec<FoundText> {
        let (folded_text, boundaries) = fold_text_with_boundaries(text);
        folded_text
            .match_indices(&self.folded_query)
            .filter_map(|(start, matched)| {
                let end = start + matched.len();
                let start_boundary = find_boundary(&boundaries, start)?;
                let end_boundary = find_boundary(&boundaries, end)?;
                Some(FoundText {
                    start_byte: start_boundary.original_byte,
                    end_byte: end_boundary.original_byte,
                    start_utf16: start_boundary.original_utf16,
                    end_utf16: end_boundary.original_utf16,
                })
            })
            .collect()
    }
}

fn fold_case(value: &str) -> String {
    value.chars().flat_map(char::to_lowercase).collect()
}

fn fold_text_with_boundaries(text: &str) -> (String, Vec<TextBoundary>) {
    let mut folded = String::new();
    let mut boundaries = vec![TextBoundary {
        folded_byte: 0,
        original_byte: 0,
        original_utf16: 0,
    }];
    let mut original_utf16 = 0;

    for (original_byte, character) in text.char_indices() {
        folded.extend(character.to_lowercase());
        original_utf16 += character.len_utf16();
        boundaries.push(TextBoundary {
            folded_byte: folded.len(),
            original_byte: original_byte + character.len_utf8(),
            original_utf16,
        });
    }

    (folded, boundaries)
}

fn find_boundary(boundaries: &[TextBoundary], folded_byte: usize) -> Option<TextBoundary> {
    boundaries
        .binary_search_by_key(&folded_byte, |boundary| boundary.folded_byte)
        .ok()
        .map(|index| boundaries[index])
}

fn create_snippet(text: &str, match_start: usize, match_end: usize) -> String {
    let prefix = &text[..match_start];
    let suffix = &text[match_end..];
    let prefix_char_count = prefix.chars().count();
    let suffix_char_count = suffix.chars().count();
    let visible_prefix: String = prefix
        .chars()
        .rev()
        .take(SNIPPET_CONTEXT_CHARACTERS)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let visible_suffix: String = suffix.chars().take(SNIPPET_CONTEXT_CHARACTERS).collect();
    let leading_ellipsis = if prefix_char_count > SNIPPET_CONTEXT_CHARACTERS {
        "…"
    } else {
        ""
    };
    let trailing_ellipsis = if suffix_char_count > SNIPPET_CONTEXT_CHARACTERS {
        "…"
    } else {
        ""
    };
    let snippet = format!(
        "{leading_ellipsis}{visible_prefix}{}{visible_suffix}{trailing_ellipsis}",
        &text[match_start..match_end]
    );
    snippet.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn database_error(path: &std::path::Path, source: rusqlite::Error) -> SearchProjectError {
    SearchProjectError::Database {
        path: path.to_owned(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io};

    use rusqlite::params;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::application::{
        create_document::{create_document, CreateDocumentInput, CreatedDocument},
        create_project::{create_project, CreateProjectInput},
        document_trash::{trash_document, TrashDocumentInput},
    };

    struct TestProject {
        parent: PathBuf,
        path: PathBuf,
    }

    impl TestProject {
        fn create() -> Self {
            let parent = std::env::temp_dir()
                .join(format!("bookmaker-project-search-test-{}", Uuid::now_v7()));
            fs::create_dir(&parent).expect("test parent must be created");
            let path = parent.join("Project.bookmaker");
            create_project(CreateProjectInput {
                destination: path.clone(),
                title: "Livro".to_owned(),
                language: "pt-BR".to_owned(),
            })
            .expect("test project must be created");
            Self { parent, path }
        }

        fn create_document(&self, title: &str) -> CreatedDocument {
            create_document(CreateDocumentInput {
                project_path: self.path.clone(),
                title: title.to_owned(),
                role: None,
            })
            .expect("test document must be created")
        }

        fn set_content(&self, document_id: &str, content: Value) {
            self.connection()
                .execute(
                    "UPDATE document_content SET json_content = ?1 WHERE document_id = ?2",
                    params![content.to_string(), document_id],
                )
                .expect("test content must be updated");
        }

        fn search(&self, query: &str) -> Result<ProjectSearchResults, SearchProjectError> {
            search_project(SearchProjectInput {
                project_path: self.path.clone(),
                query: query.to_owned(),
            })
        }

        fn connection(&self) -> Connection {
            Connection::open(self.path.join(DATABASE_FILE)).expect("test database must open")
        }
    }

    impl Drop for TestProject {
        fn drop(&mut self) {
            if let Err(error) = fs::remove_dir_all(&self.parent) {
                if error.kind() != io::ErrorKind::NotFound {
                    panic!("test project cleanup failed: {error}");
                }
            }
        }
    }

    #[test]
    fn returns_empty_results_for_an_empty_query_without_opening_a_project() {
        let results = search_project(SearchProjectInput {
            project_path: PathBuf::from("missing.bookmaker"),
            query: String::new(),
        })
        .expect("empty search must not access the project");

        assert_eq!(
            results,
            ProjectSearchResults {
                matches: vec![],
                truncated: false,
            }
        );
    }

    #[test]
    fn searches_all_active_documents_with_utf16_editor_positions() {
        let project = TestProject::create();
        let first = project.create_document("Primeiro capítulo");
        let second = project.create_document("Segundo capítulo");
        project.set_content(
            &first.document_id,
            json!({
                "type": "doc",
                "content": [{
                    "type": "paragraph",
                    "content": [
                        {"type": "text", "text": "😀 Book", "marks": [{"type": "bold"}]},
                        {"type": "text", "text": "Maker encontra outro bookmaker."}
                    ]
                }]
            }),
        );
        project.set_content(
            &second.document_id,
            json!({
                "type": "doc",
                "content": [{
                    "type": "paragraph",
                    "content": [{"type": "text", "text": "BOOKMAKER final."}]
                }]
            }),
        );

        let results = project.search("bookmaker").unwrap();

        assert_eq!(results.matches.len(), 3);
        assert!(!results.truncated);
        assert_eq!(results.matches[0].document_id, first.document_id);
        assert_eq!(results.matches[0].document_title, "Primeiro capítulo");
        assert_eq!((results.matches[0].from, results.matches[0].to), (4, 13));
        assert!(results.matches[0].snippet.contains("BookMaker"));
        assert_eq!(results.matches[2].document_id, second.document_id);
    }

    #[test]
    fn treats_special_characters_literally_and_keeps_accents_significant() {
        let project = TestProject::create();
        let document = project.create_document("Capítulo");
        project.set_content(
            &document.document_id,
            json!({
                "type": "doc",
                "content": [{
                    "type": "paragraph",
                    "content": [{"type": "text", "text": "(literal) capítulo capitulo"}]
                }]
            }),
        );

        assert_eq!(project.search("(literal)").unwrap().matches.len(), 1);
        assert_eq!(project.search("capítulo").unwrap().matches.len(), 1);
        assert_eq!(project.search("capitulo").unwrap().matches.len(), 1);
    }

    #[test]
    fn excludes_documents_in_the_internal_trash() {
        let project = TestProject::create();
        let visible = project.create_document("Visível");
        let trashed = project.create_document("Lixeira");
        for document in [&visible, &trashed] {
            project.set_content(
                &document.document_id,
                json!({
                    "type": "doc",
                    "content": [{
                        "type": "paragraph",
                        "content": [{"type": "text", "text": "agulha"}]
                    }]
                }),
            );
        }
        trash_document(TrashDocumentInput {
            project_path: project.path.clone(),
            document_id: trashed.document_id,
        })
        .unwrap();

        let results = project.search("agulha").unwrap();

        assert_eq!(results.matches.len(), 1);
        assert_eq!(results.matches[0].document_id, visible.document_id);
    }

    #[test]
    fn reports_unsupported_editor_content_versions() {
        let project = TestProject::create();
        let document = project.create_document("Futuro");
        project
            .connection()
            .execute(
                "UPDATE document_content SET schema_version = 2 WHERE document_id = ?1",
                [&document.document_id],
            )
            .unwrap();

        let error = project.search("texto").unwrap_err();

        assert!(matches!(
            error,
            SearchProjectError::UnsupportedContentSchema {
                schema_version: 2,
                ..
            }
        ));
    }

    #[test]
    fn caps_the_response_and_reports_truncation() {
        let project = TestProject::create();
        let document = project.create_document("Longo");
        let text = std::iter::repeat_n("x", MAX_RESULTS + 1)
            .collect::<Vec<_>>()
            .join(" ");
        project.set_content(
            &document.document_id,
            json!({
                "type": "doc",
                "content": [{
                    "type": "paragraph",
                    "content": [{"type": "text", "text": text}]
                }]
            }),
        );

        let results = project.search("x").unwrap();

        assert_eq!(results.matches.len(), MAX_RESULTS);
        assert!(results.truncated);
    }
}
