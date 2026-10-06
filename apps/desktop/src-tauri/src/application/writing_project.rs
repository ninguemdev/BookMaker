use std::{error::Error, fmt, path::PathBuf};

use rusqlite::{params, Connection, OpenFlags, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    application::open_project::{open_project, OpenProjectError, OpenProjectInput},
    persistence::project_manifest::DATABASE_FILE,
};

const EDITOR_SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadWritingProjectInput {
    pub project_path: PathBuf,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WritingDocument {
    pub document_id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub position: i64,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WritingProject {
    pub project_id: String,
    pub path: PathBuf,
    pub title: String,
    pub language: String,
    pub documents: Vec<WritingDocument>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadDocumentContentInput {
    pub project_path: PathBuf,
    pub document_id: String,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedDocumentContent {
    pub document_id: String,
    pub schema_version: i64,
    pub content: Value,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDocumentContentInput {
    pub project_path: PathBuf,
    pub document_id: String,
    pub schema_version: i64,
    pub content: Value,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedDocumentContent {
    pub document_id: String,
    pub updated_at: String,
}

#[derive(Debug)]
pub enum WritingProjectError {
    InvalidDocumentId,
    DocumentNotFound,
    UnsupportedContentSchema {
        schema_version: i64,
    },
    InvalidContent(&'static str),
    OpenProject(OpenProjectError),
    Database {
        path: PathBuf,
        source: rusqlite::Error,
    },
    InvalidPersistedContent {
        document_id: String,
        source: serde_json::Error,
    },
}

impl WritingProjectError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidDocumentId => "document.invalid_id",
            Self::DocumentNotFound => "document.not_found",
            Self::UnsupportedContentSchema { .. } => "document.unsupported_content",
            Self::InvalidContent(_) | Self::InvalidPersistedContent { .. } => {
                "document.invalid_content"
            }
            Self::OpenProject(source) => source.code(),
            Self::Database { .. } => "document.content_failed",
        }
    }

    pub fn user_message(&self) -> &'static str {
        match self {
            Self::InvalidDocumentId | Self::DocumentNotFound => "O documento não foi encontrado.",
            Self::UnsupportedContentSchema { .. } => {
                "O documento usa uma versão de conteúdo ainda não suportada."
            }
            Self::InvalidContent(_) | Self::InvalidPersistedContent { .. } => {
                "Os dados do documento estão inconsistentes."
            }
            Self::OpenProject(source) => source.user_message(),
            Self::Database { .. } => "Não foi possível acessar o conteúdo do documento.",
        }
    }
}

impl fmt::Display for WritingProjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDocumentId => formatter.write_str("document ID cannot be blank"),
            Self::DocumentNotFound => formatter.write_str("active flow document was not found"),
            Self::UnsupportedContentSchema { schema_version } => write!(
                formatter,
                "editor content schema {schema_version} is not supported"
            ),
            Self::InvalidContent(reason) => write!(formatter, "invalid editor content: {reason}"),
            Self::OpenProject(source) => write!(formatter, "cannot open project: {source}"),
            Self::Database { path, source } => write!(
                formatter,
                "document content database operation failed at {}: {source}",
                path.display()
            ),
            Self::InvalidPersistedContent {
                document_id,
                source,
            } => write!(
                formatter,
                "document {document_id} contains invalid JSON content: {source}"
            ),
        }
    }
}

impl Error for WritingProjectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OpenProject(source) => Some(source),
            Self::Database { source, .. } => Some(source),
            Self::InvalidPersistedContent { source, .. } => Some(source),
            Self::InvalidDocumentId
            | Self::DocumentNotFound
            | Self::UnsupportedContentSchema { .. }
            | Self::InvalidContent(_) => None,
        }
    }
}

pub fn load_writing_project(
    input: LoadWritingProjectInput,
) -> Result<WritingProject, WritingProjectError> {
    let project = open_project(OpenProjectInput {
        path: input.project_path,
    })
    .map_err(WritingProjectError::OpenProject)?;
    let database_path = project.path.join(DATABASE_FILE);
    let connection = open_database(&database_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let documents = load_documents(&connection, &database_path)?;

    Ok(WritingProject {
        project_id: project.project_id,
        path: project.path,
        title: project.title,
        language: project.language,
        documents,
    })
}

pub fn load_document_content(
    input: LoadDocumentContentInput,
) -> Result<LoadedDocumentContent, WritingProjectError> {
    let document_id = validate_document_id(&input.document_id)?;
    let project = open_project(OpenProjectInput {
        path: input.project_path,
    })
    .map_err(WritingProjectError::OpenProject)?;
    let database_path = project.path.join(DATABASE_FILE);
    let connection = open_database(&database_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let persisted = connection
        .query_row(
            "SELECT document_content.schema_version,
                    document_content.json_content,
                    document_content.updated_at
             FROM document_content
             JOIN documents ON documents.id = document_content.document_id
             WHERE documents.id = ?1
               AND documents.kind = 'flow'
               AND documents.status <> 'trashed'",
            [&document_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()
        .map_err(|source| database_error(&database_path, source))?
        .ok_or(WritingProjectError::DocumentNotFound)?;

    ensure_supported_schema(persisted.0)?;
    let content: Value = serde_json::from_str(&persisted.1).map_err(|source| {
        WritingProjectError::InvalidPersistedContent {
            document_id: document_id.clone(),
            source,
        }
    })?;
    validate_content_root(&content)?;

    Ok(LoadedDocumentContent {
        document_id,
        schema_version: persisted.0,
        content,
        updated_at: persisted.2,
    })
}

pub fn save_document_content(
    input: SaveDocumentContentInput,
) -> Result<SavedDocumentContent, WritingProjectError> {
    let document_id = validate_document_id(&input.document_id)?;
    ensure_supported_schema(input.schema_version)?;
    validate_content_root(&input.content)?;
    let json_content = serde_json::to_string(&input.content)
        .expect("serializing an existing JSON value must succeed");
    let project = open_project(OpenProjectInput {
        path: input.project_path,
    })
    .map_err(WritingProjectError::OpenProject)?;
    let database_path = project.path.join(DATABASE_FILE);
    let mut connection = open_database(&database_path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|source| database_error(&database_path, source))?;
    let updated_at: String = transaction
        .query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ', 'now')", [], |row| {
            row.get(0)
        })
        .map_err(|source| database_error(&database_path, source))?;
    let updated = transaction
        .execute(
            "UPDATE document_content
             SET schema_version = ?2, json_content = ?3, updated_at = ?4
             WHERE document_id = ?1
               AND EXISTS (
                   SELECT 1 FROM documents
                   WHERE documents.id = document_content.document_id
                     AND documents.kind = 'flow'
                     AND documents.status <> 'trashed'
               )",
            params![document_id, input.schema_version, json_content, updated_at],
        )
        .map_err(|source| database_error(&database_path, source))?;
    if updated == 0 {
        return Err(WritingProjectError::DocumentNotFound);
    }
    transaction
        .execute("UPDATE project_info SET updated_at = ?1", [&updated_at])
        .map_err(|source| database_error(&database_path, source))?;
    transaction
        .commit()
        .map_err(|source| database_error(&database_path, source))?;

    Ok(SavedDocumentContent {
        document_id,
        updated_at,
    })
}

fn load_documents(
    connection: &Connection,
    database_path: &std::path::Path,
) -> Result<Vec<WritingDocument>, WritingProjectError> {
    let mut statement = connection
        .prepare(
            "SELECT id, parent_id, title, position
             FROM documents
             WHERE kind = 'flow' AND status <> 'trashed'
             ORDER BY position, id",
        )
        .map_err(|source| database_error(database_path, source))?;
    let documents = statement
        .query_map([], |row| {
            Ok(WritingDocument {
                document_id: row.get(0)?,
                parent_id: row.get(1)?,
                title: row.get(2)?,
                position: row.get(3)?,
            })
        })
        .map_err(|source| database_error(database_path, source))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|source| database_error(database_path, source))?;
    Ok(documents)
}

fn open_database(
    database_path: &std::path::Path,
    flags: OpenFlags,
) -> Result<Connection, WritingProjectError> {
    Connection::open_with_flags(database_path, flags)
        .map_err(|source| database_error(database_path, source))
}

fn validate_document_id(document_id: &str) -> Result<String, WritingProjectError> {
    let document_id = document_id.trim();
    if document_id.is_empty() {
        return Err(WritingProjectError::InvalidDocumentId);
    }
    Ok(document_id.to_owned())
}

fn ensure_supported_schema(schema_version: i64) -> Result<(), WritingProjectError> {
    if schema_version != EDITOR_SCHEMA_VERSION {
        return Err(WritingProjectError::UnsupportedContentSchema { schema_version });
    }
    Ok(())
}

fn validate_content_root(content: &Value) -> Result<(), WritingProjectError> {
    let root = content
        .as_object()
        .ok_or(WritingProjectError::InvalidContent(
            "the root must be an object",
        ))?;
    if root.get("type").and_then(Value::as_str) != Some("doc") {
        return Err(WritingProjectError::InvalidContent(
            "the root must be a doc node",
        ));
    }
    if let Some(children) = root.get("content") {
        if !children.is_array() {
            return Err(WritingProjectError::InvalidContent(
                "doc content must be an array",
            ));
        }
    }
    Ok(())
}

fn database_error(path: &std::path::Path, source: rusqlite::Error) -> WritingProjectError {
    WritingProjectError::Database {
        path: path.to_owned(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io};

    use rusqlite::Connection;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::application::{
        create_document::{create_document, CreateDocumentInput},
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
                .join(format!("bookmaker-writing-project-test-{}", Uuid::now_v7()));
            fs::create_dir(&parent).expect("test parent must be created");
            let path = parent.join("Project.bookmaker");
            create_project(CreateProjectInput {
                destination: path.clone(),
                title: "Meu Livro".to_owned(),
                language: "pt-BR".to_owned(),
            })
            .expect("test project must be created");
            Self { parent, path }
        }

        fn create_document(&self, title: &str) -> String {
            create_document(CreateDocumentInput {
                project_path: self.path.clone(),
                title: title.to_owned(),
                role: None,
            })
            .expect("test document must be created")
            .document_id
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
    fn loads_the_project_and_only_active_flow_documents() {
        let project = TestProject::create();
        let first_id = project.create_document("Capítulo 1");
        let trashed_id = project.create_document("Rascunho removido");
        trash_document(TrashDocumentInput {
            project_path: project.path.clone(),
            document_id: trashed_id,
        })
        .unwrap();

        let loaded = load_writing_project(LoadWritingProjectInput {
            project_path: project.path.clone(),
        })
        .unwrap();

        assert_eq!(loaded.title, "Meu Livro");
        assert_eq!(loaded.language, "pt-BR");
        assert_eq!(loaded.documents.len(), 1);
        assert_eq!(loaded.documents[0].document_id, first_id);
        assert_eq!(loaded.documents[0].title, "Capítulo 1");
    }

    #[test]
    fn saves_and_reloads_unicode_editor_content() {
        let project = TestProject::create();
        let document_id = project.create_document("Capítulo");
        let content = json!({
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "content": [{"type": "text", "text": "Névoa — café ☕"}]
            }]
        });

        let saved = save_document_content(SaveDocumentContentInput {
            project_path: project.path.clone(),
            document_id: document_id.clone(),
            schema_version: EDITOR_SCHEMA_VERSION,
            content: content.clone(),
        })
        .unwrap();
        let loaded = load_document_content(LoadDocumentContentInput {
            project_path: project.path.clone(),
            document_id: document_id.clone(),
        })
        .unwrap();

        assert_eq!(saved.document_id, document_id);
        assert_eq!(loaded.content, content);
        assert_eq!(loaded.updated_at, saved.updated_at);
        let project_updated_at: String = project
            .connection()
            .query_row("SELECT updated_at FROM project_info", [], |row| row.get(0))
            .unwrap();
        assert_eq!(project_updated_at, saved.updated_at);
    }

    #[test]
    fn refuses_to_load_or_save_a_trashed_document() {
        let project = TestProject::create();
        let document_id = project.create_document("Capítulo");
        trash_document(TrashDocumentInput {
            project_path: project.path.clone(),
            document_id: document_id.clone(),
        })
        .unwrap();

        let load_error = load_document_content(LoadDocumentContentInput {
            project_path: project.path.clone(),
            document_id: document_id.clone(),
        })
        .unwrap_err();
        let save_error = save_document_content(SaveDocumentContentInput {
            project_path: project.path.clone(),
            document_id,
            schema_version: EDITOR_SCHEMA_VERSION,
            content: json!({"type": "doc", "content": []}),
        })
        .unwrap_err();

        assert!(matches!(load_error, WritingProjectError::DocumentNotFound));
        assert!(matches!(save_error, WritingProjectError::DocumentNotFound));
    }

    #[test]
    fn rejects_unsupported_schemas_and_invalid_roots_before_writing() {
        let project = TestProject::create();
        let document_id = project.create_document("Capítulo");

        let schema_error = save_document_content(SaveDocumentContentInput {
            project_path: project.path.clone(),
            document_id: document_id.clone(),
            schema_version: 2,
            content: json!({"type": "doc", "content": []}),
        })
        .unwrap_err();
        let content_error = save_document_content(SaveDocumentContentInput {
            project_path: project.path.clone(),
            document_id,
            schema_version: EDITOR_SCHEMA_VERSION,
            content: json!({"type": "paragraph"}),
        })
        .unwrap_err();

        assert!(matches!(
            schema_error,
            WritingProjectError::UnsupportedContentSchema { schema_version: 2 }
        ));
        assert!(matches!(
            content_error,
            WritingProjectError::InvalidContent(_)
        ));
    }
}
