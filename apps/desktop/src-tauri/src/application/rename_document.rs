use std::{error::Error, fmt, path::PathBuf};

use rusqlite::{params, Connection, OpenFlags, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};

use crate::{
    application::open_project::{open_project, OpenProjectError, OpenProjectInput},
    persistence::project_manifest::DATABASE_FILE,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameDocumentInput {
    pub project_path: PathBuf,
    pub document_id: String,
    pub title: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenamedDocument {
    pub document_id: String,
    pub title: String,
    pub updated_at: String,
}

#[derive(Debug)]
pub enum RenameDocumentError {
    InvalidDocumentId,
    InvalidTitle,
    DocumentNotFound,
    OpenProject(OpenProjectError),
    Database {
        path: PathBuf,
        source: rusqlite::Error,
    },
}

impl RenameDocumentError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidDocumentId => "document.invalid_id",
            Self::InvalidTitle => "document.invalid_title",
            Self::DocumentNotFound => "document.not_found",
            Self::OpenProject(source) => source.code(),
            Self::Database { .. } => "document.rename_failed",
        }
    }

    pub fn user_message(&self) -> &'static str {
        match self {
            Self::InvalidDocumentId | Self::DocumentNotFound => "O documento não foi encontrado.",
            Self::InvalidTitle => "Informe um título para o documento.",
            Self::OpenProject(source) => source.user_message(),
            Self::Database { .. } => "Não foi possível renomear o documento.",
        }
    }
}

impl fmt::Display for RenameDocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDocumentId => formatter.write_str("document ID cannot be blank"),
            Self::InvalidTitle => formatter.write_str("document title cannot be blank"),
            Self::DocumentNotFound => formatter.write_str("document was not found"),
            Self::OpenProject(source) => write!(formatter, "cannot open project: {source}"),
            Self::Database { path, source } => write!(
                formatter,
                "document database operation failed at {}: {source}",
                path.display()
            ),
        }
    }
}

impl Error for RenameDocumentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OpenProject(source) => Some(source),
            Self::Database { source, .. } => Some(source),
            Self::InvalidDocumentId | Self::InvalidTitle | Self::DocumentNotFound => None,
        }
    }
}

pub fn rename_document(input: RenameDocumentInput) -> Result<RenamedDocument, RenameDocumentError> {
    let document_id = validate_document_id(&input.document_id)?;
    let title = validate_title(&input.title)?;
    let project = open_project(OpenProjectInput {
        path: input.project_path,
    })
    .map_err(RenameDocumentError::OpenProject)?;
    let database_path = project.path.join(DATABASE_FILE);
    let mut connection =
        Connection::open_with_flags(&database_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
            .map_err(|source| database_error(&database_path, source))?;
    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(|source| database_error(&database_path, source))?;

    update_document_title(&mut connection, document_id, title)
        .map_err(|source| database_error(&database_path, source))?
        .ok_or(RenameDocumentError::DocumentNotFound)
}

fn validate_document_id(document_id: &str) -> Result<String, RenameDocumentError> {
    let document_id = document_id.trim();
    if document_id.is_empty() {
        return Err(RenameDocumentError::InvalidDocumentId);
    }
    Ok(document_id.to_owned())
}

fn validate_title(title: &str) -> Result<String, RenameDocumentError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(RenameDocumentError::InvalidTitle);
    }
    Ok(title.to_owned())
}

fn update_document_title(
    connection: &mut Connection,
    document_id: String,
    title: String,
) -> rusqlite::Result<Option<RenamedDocument>> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let renamed = transaction
        .query_row(
            "UPDATE documents
             SET title = ?2,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?1
             RETURNING title, updated_at",
            params![document_id, title],
            |row| {
                Ok(RenamedDocument {
                    document_id: document_id.clone(),
                    title: row.get(0)?,
                    updated_at: row.get(1)?,
                })
            },
        )
        .optional()?;

    let Some(renamed) = renamed else {
        return Ok(None);
    };

    transaction.execute(
        "UPDATE project_info SET updated_at = ?1",
        [&renamed.updated_at],
    )?;
    transaction.commit()?;

    Ok(Some(renamed))
}

fn database_error(path: &std::path::Path, source: rusqlite::Error) -> RenameDocumentError {
    RenameDocumentError::Database {
        path: path.to_owned(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io};

    use uuid::Uuid;

    use super::*;
    use crate::application::{
        create_document::{create_document, CreateDocumentInput, CreatedDocument},
        create_project::{create_project, CreateProjectInput},
    };

    struct TestProject {
        parent: PathBuf,
        path: PathBuf,
    }

    impl TestProject {
        fn create() -> Self {
            let parent = std::env::temp_dir()
                .join(format!("bookmaker-rename-document-test-{}", Uuid::now_v7()));
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
    fn renames_a_document_and_updates_structural_timestamps() {
        let project = TestProject::create();
        let document = project.create_document("Rascunho");
        let connection = project.connection();
        connection
            .execute(
                "UPDATE documents SET updated_at = '2000-01-01T00:00:00.000Z' WHERE id = ?1",
                [&document.document_id],
            )
            .unwrap();
        connection
            .execute(
                "UPDATE project_info SET updated_at = '2000-01-01T00:00:00.000Z'",
                [],
            )
            .unwrap();
        let content_updated_at: String = connection
            .query_row(
                "SELECT updated_at FROM document_content WHERE document_id = ?1",
                [&document.document_id],
                |row| row.get(0),
            )
            .unwrap();
        drop(connection);

        let renamed = rename_document(RenameDocumentInput {
            project_path: project.path.clone(),
            document_id: format!("  {}  ", document.document_id),
            title: "  Capítulo — Café  ".to_owned(),
        })
        .expect("document must be renamed");

        assert_eq!(renamed.document_id, document.document_id);
        assert_eq!(renamed.title, "Capítulo — Café");
        assert_ne!(renamed.updated_at, "2000-01-01T00:00:00.000Z");

        let connection = project.connection();
        let persisted: (String, String) = connection
            .query_row(
                "SELECT title, updated_at FROM documents WHERE id = ?1",
                [&renamed.document_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let project_updated_at: String = connection
            .query_row("SELECT updated_at FROM project_info", [], |row| row.get(0))
            .unwrap();
        let persisted_content_updated_at: String = connection
            .query_row(
                "SELECT updated_at FROM document_content WHERE document_id = ?1",
                [&renamed.document_id],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(persisted, (renamed.title, renamed.updated_at.clone()));
        assert_eq!(project_updated_at, renamed.updated_at);
        assert_eq!(persisted_content_updated_at, content_updated_at);
    }

    #[test]
    fn rejects_a_blank_title_before_accessing_the_project() {
        let error = rename_document(RenameDocumentInput {
            project_path: PathBuf::from("missing.bookmaker"),
            document_id: "document-1".to_owned(),
            title: "   ".to_owned(),
        })
        .expect_err("blank titles must be rejected");

        assert!(matches!(error, RenameDocumentError::InvalidTitle));
    }

    #[test]
    fn rejects_a_blank_document_id_before_accessing_the_project() {
        let error = rename_document(RenameDocumentInput {
            project_path: PathBuf::from("missing.bookmaker"),
            document_id: "   ".to_owned(),
            title: "Capítulo".to_owned(),
        })
        .expect_err("blank document IDs must be rejected");

        assert!(matches!(error, RenameDocumentError::InvalidDocumentId));
    }

    #[test]
    fn reports_when_the_document_does_not_exist() {
        let project = TestProject::create();

        let error = rename_document(RenameDocumentInput {
            project_path: project.path.clone(),
            document_id: Uuid::now_v7().to_string(),
            title: "Capítulo".to_owned(),
        })
        .expect_err("a missing document must be reported");

        assert!(matches!(error, RenameDocumentError::DocumentNotFound));
    }

    #[test]
    fn rolls_back_the_title_when_the_project_timestamp_cannot_be_updated() {
        let project = TestProject::create();
        let document = project.create_document("Título original");
        let connection = project.connection();
        connection
            .execute_batch(
                "CREATE TRIGGER reject_project_update
                 BEFORE UPDATE ON project_info
                 BEGIN
                   SELECT RAISE(ABORT, 'simulated project update failure');
                 END;",
            )
            .unwrap();
        drop(connection);

        let error = rename_document(RenameDocumentInput {
            project_path: project.path.clone(),
            document_id: document.document_id.clone(),
            title: "Novo título".to_owned(),
        })
        .expect_err("project update failure must abort the rename");

        assert!(matches!(error, RenameDocumentError::Database { .. }));
        let persisted_title: String = project
            .connection()
            .query_row(
                "SELECT title FROM documents WHERE id = ?1",
                [&document.document_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(persisted_title, "Título original");
    }
}
