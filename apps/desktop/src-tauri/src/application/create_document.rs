use std::{error::Error, fmt, path::PathBuf};

use rusqlite::{params, Connection, OpenFlags, TransactionBehavior};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    application::open_project::{open_project, OpenProjectError, OpenProjectInput},
    persistence::project_manifest::DATABASE_FILE,
};

const FLOW_DOCUMENT_KIND: &str = "flow";
const ACTIVE_DOCUMENT_STATUS: &str = "active";
const INITIAL_CONTENT_SCHEMA_VERSION: i64 = 1;
const INITIAL_FLOW_CONTENT: &str = r#"{"type":"doc","content":[]}"#;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDocumentInput {
    pub project_path: PathBuf,
    pub title: String,
    #[serde(default)]
    pub role: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedDocument {
    pub document_id: String,
    pub parent_id: Option<String>,
    pub kind: &'static str,
    pub role: &'static str,
    pub section: &'static str,
    pub title: String,
    pub position: i64,
    pub status: &'static str,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug)]
pub enum CreateDocumentError {
    InvalidTitle,
    InvalidRole,
    OpenProject(OpenProjectError),
    Database {
        path: PathBuf,
        source: rusqlite::Error,
    },
}

impl CreateDocumentError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidTitle => "document.invalid_title",
            Self::InvalidRole => "document.invalid_role",
            Self::OpenProject(source) => source.code(),
            Self::Database { .. } => "document.create_failed",
        }
    }

    pub fn user_message(&self) -> &'static str {
        match self {
            Self::InvalidTitle => "Informe um título para o documento.",
            Self::InvalidRole => "Escolha um tipo de documento válido.",
            Self::OpenProject(source) => source.user_message(),
            Self::Database { .. } => "Não foi possível criar o documento.",
        }
    }
}

impl fmt::Display for CreateDocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTitle => formatter.write_str("document title cannot be blank"),
            Self::InvalidRole => formatter.write_str("flow document role is not supported"),
            Self::OpenProject(source) => write!(formatter, "cannot open project: {source}"),
            Self::Database { path, source } => write!(
                formatter,
                "document database operation failed at {}: {source}",
                path.display()
            ),
        }
    }
}

impl Error for CreateDocumentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OpenProject(source) => Some(source),
            Self::Database { source, .. } => Some(source),
            Self::InvalidTitle | Self::InvalidRole => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlowDocumentRole {
    Part,
    Chapter,
    Section,
    TitlePage,
    CopyrightPage,
    Dedication,
    Epigraph,
    Toc,
    Preface,
    Introduction,
    Appendix,
    Acknowledgements,
    AboutAuthor,
    CustomFrontMatter,
    CustomBackMatter,
}

impl FlowDocumentRole {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "part" => Some(Self::Part),
            "chapter" => Some(Self::Chapter),
            "section" => Some(Self::Section),
            "titlePage" => Some(Self::TitlePage),
            "copyrightPage" => Some(Self::CopyrightPage),
            "dedication" => Some(Self::Dedication),
            "epigraph" => Some(Self::Epigraph),
            "toc" => Some(Self::Toc),
            "preface" => Some(Self::Preface),
            "introduction" => Some(Self::Introduction),
            "appendix" => Some(Self::Appendix),
            "acknowledgements" => Some(Self::Acknowledgements),
            "aboutAuthor" => Some(Self::AboutAuthor),
            "customFrontMatter" => Some(Self::CustomFrontMatter),
            "customBackMatter" => Some(Self::CustomBackMatter),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Part => "part",
            Self::Chapter => "chapter",
            Self::Section => "section",
            Self::TitlePage => "titlePage",
            Self::CopyrightPage => "copyrightPage",
            Self::Dedication => "dedication",
            Self::Epigraph => "epigraph",
            Self::Toc => "toc",
            Self::Preface => "preface",
            Self::Introduction => "introduction",
            Self::Appendix => "appendix",
            Self::Acknowledgements => "acknowledgements",
            Self::AboutAuthor => "aboutAuthor",
            Self::CustomFrontMatter => "customFrontMatter",
            Self::CustomBackMatter => "customBackMatter",
        }
    }

    fn section(self) -> &'static str {
        match self {
            Self::Part | Self::Chapter | Self::Section => "manuscript",
            Self::TitlePage
            | Self::CopyrightPage
            | Self::Dedication
            | Self::Epigraph
            | Self::Toc
            | Self::Preface
            | Self::Introduction
            | Self::CustomFrontMatter => "frontMatter",
            Self::Appendix
            | Self::Acknowledgements
            | Self::AboutAuthor
            | Self::CustomBackMatter => "backMatter",
        }
    }
}

pub fn create_document(input: CreateDocumentInput) -> Result<CreatedDocument, CreateDocumentError> {
    let title = validate_title(&input.title)?;
    let role = validate_role(input.role.as_deref())?;
    let project = open_project(OpenProjectInput {
        path: input.project_path,
    })
    .map_err(CreateDocumentError::OpenProject)?;
    let database_path = project.path.join(DATABASE_FILE);
    let mut connection =
        Connection::open_with_flags(&database_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
            .map_err(|source| database_error(&database_path, source))?;
    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(|source| database_error(&database_path, source))?;

    insert_document(&mut connection, Uuid::now_v7().to_string(), title, role)
        .map_err(|source| database_error(&database_path, source))
}

fn validate_title(title: &str) -> Result<String, CreateDocumentError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(CreateDocumentError::InvalidTitle);
    }
    Ok(title.to_owned())
}

fn validate_role(role: Option<&str>) -> Result<FlowDocumentRole, CreateDocumentError> {
    let Some(role) = role else {
        return Ok(FlowDocumentRole::Chapter);
    };
    FlowDocumentRole::parse(role.trim()).ok_or(CreateDocumentError::InvalidRole)
}

fn insert_document(
    connection: &mut Connection,
    document_id: String,
    title: String,
    role: FlowDocumentRole,
) -> rusqlite::Result<CreatedDocument> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let position = transaction.query_row(
        "SELECT COALESCE(MAX(position) + 1, 0)
         FROM documents
         WHERE parent_id IS NULL",
        [],
        |row| row.get(0),
    )?;
    let (created_at, updated_at): (String, String) = transaction.query_row(
        "INSERT INTO documents (
            id, parent_id, kind, role, title, position, status, created_at, updated_at
         ) VALUES (
            ?1, NULL, ?2, ?3, ?4, ?5, ?6,
            strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
            strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         )
         RETURNING created_at, updated_at",
        params![
            document_id,
            FLOW_DOCUMENT_KIND,
            role.as_str(),
            title,
            position,
            ACTIVE_DOCUMENT_STATUS,
        ],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    transaction.execute(
        "INSERT INTO document_content (
            document_id, schema_version, json_content, updated_at
         ) VALUES (?1, ?2, ?3, ?4)",
        params![
            document_id,
            INITIAL_CONTENT_SCHEMA_VERSION,
            INITIAL_FLOW_CONTENT,
            updated_at,
        ],
    )?;
    transaction.execute("UPDATE project_info SET updated_at = ?1", [&updated_at])?;
    transaction.commit()?;

    Ok(CreatedDocument {
        document_id,
        parent_id: None,
        kind: FLOW_DOCUMENT_KIND,
        role: role.as_str(),
        section: role.section(),
        title,
        position,
        status: ACTIVE_DOCUMENT_STATUS,
        created_at,
        updated_at,
    })
}

fn database_error(path: &std::path::Path, source: rusqlite::Error) -> CreateDocumentError {
    CreateDocumentError::Database {
        path: path.to_owned(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io};

    use serde_json::{json, Value};

    use super::*;
    use crate::application::create_project::{create_project, CreateProjectInput};

    struct TestProject {
        parent: PathBuf,
        path: PathBuf,
    }

    impl TestProject {
        fn create() -> Self {
            let parent = std::env::temp_dir()
                .join(format!("bookmaker-create-document-test-{}", Uuid::now_v7()));
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
    fn creates_a_root_flow_document_with_empty_content() {
        let project = TestProject::create();

        let created = create_document(CreateDocumentInput {
            project_path: project.path.clone(),
            title: "  Capítulo — Café  ".to_owned(),
            role: None,
        })
        .expect("document must be created");

        assert_eq!(created.parent_id, None);
        assert_eq!(created.kind, "flow");
        assert_eq!(created.role, "chapter");
        assert_eq!(created.section, "manuscript");
        assert_eq!(created.title, "Capítulo — Café");
        assert_eq!(created.position, 0);
        assert_eq!(created.status, "active");
        assert_eq!(
            Uuid::parse_str(&created.document_id)
                .expect("document ID must be a UUID")
                .get_version_num(),
            7
        );

        let connection = project.connection();
        let persisted: (String, String, String, i64, String, i64, String) = connection
            .query_row(
                "SELECT documents.kind, documents.role, documents.title,
                        documents.position, documents.status,
                        document_content.schema_version, document_content.json_content
                 FROM documents
                 JOIN document_content ON document_content.document_id = documents.id
                 WHERE documents.id = ?1",
                [&created.document_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )
            .expect("document and content must be persisted");
        assert_eq!(persisted.0, "flow");
        assert_eq!(persisted.1, "chapter");
        assert_eq!(persisted.2, "Capítulo — Café");
        assert_eq!(persisted.3, 0);
        assert_eq!(persisted.4, "active");
        assert_eq!(persisted.5, INITIAL_CONTENT_SCHEMA_VERSION);
        assert_eq!(
            serde_json::from_str::<Value>(&persisted.6).unwrap(),
            json!({"type": "doc", "content": []})
        );
    }

    #[test]
    fn appends_new_documents_to_the_root_order() {
        let project = TestProject::create();

        let first = create_document(CreateDocumentInput {
            project_path: project.path.clone(),
            title: "Primeiro".to_owned(),
            role: None,
        })
        .unwrap();
        let second = create_document(CreateDocumentInput {
            project_path: project.path.clone(),
            title: "Segundo".to_owned(),
            role: None,
        })
        .unwrap();

        assert_eq!(first.position, 0);
        assert_eq!(second.position, 1);
    }

    #[test]
    fn creates_every_supported_role_in_its_editorial_section() {
        let project = TestProject::create();
        let roles = [
            ("part", "manuscript"),
            ("chapter", "manuscript"),
            ("section", "manuscript"),
            ("titlePage", "frontMatter"),
            ("copyrightPage", "frontMatter"),
            ("dedication", "frontMatter"),
            ("epigraph", "frontMatter"),
            ("toc", "frontMatter"),
            ("preface", "frontMatter"),
            ("introduction", "frontMatter"),
            ("customFrontMatter", "frontMatter"),
            ("appendix", "backMatter"),
            ("acknowledgements", "backMatter"),
            ("aboutAuthor", "backMatter"),
            ("customBackMatter", "backMatter"),
        ];

        for (role, section) in roles {
            let created = create_document(CreateDocumentInput {
                project_path: project.path.clone(),
                title: role.to_owned(),
                role: Some(format!("  {role}  ")),
            })
            .expect("supported role must create a document");

            assert_eq!(created.role, role);
            assert_eq!(created.section, section);
            let persisted_role: String = project
                .connection()
                .query_row(
                    "SELECT role FROM documents WHERE id = ?1",
                    [&created.document_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(persisted_role, role);
        }
    }

    #[test]
    fn rejects_a_blank_title_before_accessing_the_project() {
        let error = create_document(CreateDocumentInput {
            project_path: PathBuf::from("missing.bookmaker"),
            title: "   ".to_owned(),
            role: None,
        })
        .expect_err("blank titles must be rejected");

        assert!(matches!(error, CreateDocumentError::InvalidTitle));
    }

    #[test]
    fn rejects_an_unknown_role_before_accessing_the_project() {
        let error = create_document(CreateDocumentInput {
            project_path: PathBuf::from("missing.bookmaker"),
            title: "Documento".to_owned(),
            role: Some("unknownRole".to_owned()),
        })
        .expect_err("unknown roles must be rejected");

        assert!(matches!(error, CreateDocumentError::InvalidRole));
    }

    #[test]
    fn rolls_back_the_document_when_initial_content_cannot_be_inserted() {
        let project = TestProject::create();
        let mut connection = project.connection();
        connection
            .execute_batch(
                "CREATE TRIGGER reject_document_content
                 BEFORE INSERT ON document_content
                 BEGIN
                   SELECT RAISE(ABORT, 'simulated content failure');
                 END;",
            )
            .unwrap();

        insert_document(
            &mut connection,
            Uuid::now_v7().to_string(),
            "Capítulo".to_owned(),
            FlowDocumentRole::Chapter,
        )
        .expect_err("content failure must abort document creation");

        let document_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))
            .unwrap();
        assert_eq!(document_count, 0);
    }
}
