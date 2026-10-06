use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};

use crate::{
    application::open_project::{open_project, OpenProjectError, OpenProjectInput},
    persistence::{
        internal_trash::{
            list_trashed_documents as load_trashed_documents,
            move_document_to_trash as persist_document_in_trash,
            restore_document_from_trash as persist_restored_document, InternalTrashError,
            TrashedDocument as PersistedTrashedDocument,
        },
        project_manifest::DATABASE_FILE,
    },
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashDocumentInput {
    pub project_path: PathBuf,
    pub document_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreDocumentInput {
    pub project_path: PathBuf,
    pub document_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTrashedDocumentsInput {
    pub project_path: PathBuf,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashedDocument {
    pub document_id: String,
    pub title: String,
    pub previous_status: String,
    pub trashed_at: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoredDocument {
    pub document_id: String,
    pub status: String,
}

#[derive(Debug)]
pub enum DocumentTrashError {
    InvalidDocumentId,
    DocumentNotFound,
    DocumentNotTrashed,
    OpenProject(OpenProjectError),
    TrashDatabase {
        path: PathBuf,
        source: rusqlite::Error,
    },
    RestoreDatabase {
        path: PathBuf,
        source: rusqlite::Error,
    },
    ListDatabase {
        path: PathBuf,
        source: rusqlite::Error,
    },
}

impl DocumentTrashError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidDocumentId => "document.invalid_id",
            Self::DocumentNotFound => "document.not_found",
            Self::DocumentNotTrashed => "document.not_trashed",
            Self::OpenProject(source) => source.code(),
            Self::TrashDatabase { .. } => "document.trash_failed",
            Self::RestoreDatabase { .. } => "document.restore_failed",
            Self::ListDatabase { .. } => "document.trash_list_failed",
        }
    }

    pub fn user_message(&self) -> &'static str {
        match self {
            Self::InvalidDocumentId | Self::DocumentNotFound => "O documento não foi encontrado.",
            Self::DocumentNotTrashed => "O documento não está na lixeira.",
            Self::OpenProject(source) => source.user_message(),
            Self::TrashDatabase { .. } => "Não foi possível mover o documento para a lixeira.",
            Self::RestoreDatabase { .. } => "Não foi possível restaurar o documento.",
            Self::ListDatabase { .. } => "Não foi possível carregar a lixeira.",
        }
    }
}

impl fmt::Display for DocumentTrashError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDocumentId => formatter.write_str("document ID cannot be blank"),
            Self::DocumentNotFound => formatter.write_str("document was not found"),
            Self::DocumentNotTrashed => formatter.write_str("document is not in the trash"),
            Self::OpenProject(source) => write!(formatter, "cannot open project: {source}"),
            Self::TrashDatabase { path, source } => write!(
                formatter,
                "document trash database operation failed at {}: {source}",
                path.display()
            ),
            Self::RestoreDatabase { path, source } => write!(
                formatter,
                "document restore database operation failed at {}: {source}",
                path.display()
            ),
            Self::ListDatabase { path, source } => write!(
                formatter,
                "document trash list database operation failed at {}: {source}",
                path.display()
            ),
        }
    }
}

impl Error for DocumentTrashError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OpenProject(source) => Some(source),
            Self::TrashDatabase { source, .. }
            | Self::RestoreDatabase { source, .. }
            | Self::ListDatabase { source, .. } => Some(source),
            Self::InvalidDocumentId | Self::DocumentNotFound | Self::DocumentNotTrashed => None,
        }
    }
}

#[derive(Clone, Copy)]
enum TrashOperation {
    Move,
    Restore,
    List,
}

pub fn trash_document(input: TrashDocumentInput) -> Result<TrashedDocument, DocumentTrashError> {
    let document_id = validate_document_id(&input.document_id)?;
    let (mut connection, database_path) = open_database(input.project_path, TrashOperation::Move)?;
    persist_document_in_trash(&mut connection, &document_id)
        .map(TrashedDocument::from)
        .map_err(|error| map_persistence_error(error, TrashOperation::Move, &database_path))
}

pub fn restore_document(
    input: RestoreDocumentInput,
) -> Result<RestoredDocument, DocumentTrashError> {
    let document_id = validate_document_id(&input.document_id)?;
    let (mut connection, database_path) =
        open_database(input.project_path, TrashOperation::Restore)?;
    persist_restored_document(&mut connection, &document_id)
        .map(|restored| RestoredDocument {
            document_id: restored.document_id,
            status: restored.status,
        })
        .map_err(|error| map_persistence_error(error, TrashOperation::Restore, &database_path))
}

pub fn list_trashed_documents(
    input: ListTrashedDocumentsInput,
) -> Result<Vec<TrashedDocument>, DocumentTrashError> {
    let (connection, database_path) = open_database(input.project_path, TrashOperation::List)?;
    load_trashed_documents(&connection)
        .map(|documents| documents.into_iter().map(TrashedDocument::from).collect())
        .map_err(|error| map_persistence_error(error, TrashOperation::List, &database_path))
}

impl From<PersistedTrashedDocument> for TrashedDocument {
    fn from(document: PersistedTrashedDocument) -> Self {
        Self {
            document_id: document.document_id,
            title: document.title,
            previous_status: document.previous_status,
            trashed_at: document.trashed_at,
        }
    }
}

fn validate_document_id(document_id: &str) -> Result<String, DocumentTrashError> {
    let document_id = document_id.trim();
    if document_id.is_empty() {
        return Err(DocumentTrashError::InvalidDocumentId);
    }
    Ok(document_id.to_owned())
}

fn open_database(
    project_path: PathBuf,
    operation: TrashOperation,
) -> Result<(Connection, PathBuf), DocumentTrashError> {
    let project = open_project(OpenProjectInput { path: project_path })
        .map_err(DocumentTrashError::OpenProject)?;
    let database_path = project.path.join(DATABASE_FILE);
    let connection = Connection::open_with_flags(&database_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .map_err(|source| database_error(operation, &database_path, source))?;
    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(|source| database_error(operation, &database_path, source))?;
    Ok((connection, database_path))
}

fn map_persistence_error(
    error: InternalTrashError,
    operation: TrashOperation,
    database_path: &Path,
) -> DocumentTrashError {
    match error {
        InternalTrashError::InvalidDocumentId => DocumentTrashError::InvalidDocumentId,
        InternalTrashError::DocumentNotFound(_) => DocumentTrashError::DocumentNotFound,
        InternalTrashError::DocumentNotTrashed(_) => DocumentTrashError::DocumentNotTrashed,
        InternalTrashError::Database(source) => database_error(operation, database_path, source),
    }
}

fn database_error(
    operation: TrashOperation,
    path: &Path,
    source: rusqlite::Error,
) -> DocumentTrashError {
    match operation {
        TrashOperation::Move => DocumentTrashError::TrashDatabase {
            path: path.to_owned(),
            source,
        },
        TrashOperation::Restore => DocumentTrashError::RestoreDatabase {
            path: path.to_owned(),
            source,
        },
        TrashOperation::List => DocumentTrashError::ListDatabase {
            path: path.to_owned(),
            source,
        },
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
        move_document::{move_document, MoveDocumentInput},
    };

    struct TestProject {
        parent: PathBuf,
        path: PathBuf,
    }

    impl TestProject {
        fn create() -> Self {
            let parent = std::env::temp_dir()
                .join(format!("bookmaker-document-trash-test-{}", Uuid::now_v7()));
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

        fn trash(&self, document_id: &str) -> Result<TrashedDocument, DocumentTrashError> {
            trash_document(TrashDocumentInput {
                project_path: self.path.clone(),
                document_id: document_id.to_owned(),
            })
        }

        fn restore(&self, document_id: &str) -> Result<RestoredDocument, DocumentTrashError> {
            restore_document(RestoreDocumentInput {
                project_path: self.path.clone(),
                document_id: document_id.to_owned(),
            })
        }

        fn list_trash(&self) -> Result<Vec<TrashedDocument>, DocumentTrashError> {
            list_trashed_documents(ListTrashedDocumentsInput {
                project_path: self.path.clone(),
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
    fn trashes_lists_and_restores_a_document_without_losing_its_data() {
        let project = TestProject::create();
        let parent = project.create_document("Parte");
        let document = project.create_document("Capítulo");
        move_document(MoveDocumentInput {
            project_path: project.path.clone(),
            document_id: document.document_id.clone(),
            new_parent_id: Some(parent.document_id.clone()),
            position: 0,
        })
        .unwrap();
        let connection = project.connection();
        let content_before: (i64, String, String) = connection
            .query_row(
                "SELECT schema_version, json_content, updated_at
                 FROM document_content WHERE document_id = ?1",
                [&document.document_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        drop(connection);

        let trashed = project.trash(&document.document_id).unwrap();

        assert_eq!(trashed.document_id, document.document_id);
        assert_eq!(trashed.previous_status, "active");
        assert_eq!(project.list_trash().unwrap(), vec![trashed]);
        let connection = project.connection();
        let trashed_state: (Option<String>, i64, String, String, Option<String>) = connection
            .query_row(
                "SELECT parent_id, position, status, updated_at, status_before_trash
                 FROM documents WHERE id = ?1",
                [&document.document_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .unwrap();
        let project_updated_at: String = connection
            .query_row("SELECT updated_at FROM project_info", [], |row| row.get(0))
            .unwrap();
        let content_after: (i64, String, String) = connection
            .query_row(
                "SELECT schema_version, json_content, updated_at
                 FROM document_content WHERE document_id = ?1",
                [&document.document_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(trashed_state.0, Some(parent.document_id.clone()));
        assert_eq!(trashed_state.1, 0);
        assert_eq!(trashed_state.2, "trashed");
        assert_eq!(trashed_state.3, project_updated_at);
        assert_eq!(trashed_state.4, Some("active".to_owned()));
        assert_eq!(content_after, content_before);
        drop(connection);

        let restored = project.restore(&document.document_id).unwrap();

        assert_eq!(restored.document_id, document.document_id);
        assert_eq!(restored.status, "active");
        assert!(project.list_trash().unwrap().is_empty());
        let connection = project.connection();
        let restored_state: (
            Option<String>,
            i64,
            String,
            Option<String>,
            Option<String>,
            String,
        ) = connection
            .query_row(
                "SELECT parent_id, position, status, trashed_at, status_before_trash, updated_at
                 FROM documents WHERE id = ?1",
                [&document.document_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )
            .unwrap();
        let restored_project_updated_at: String = connection
            .query_row("SELECT updated_at FROM project_info", [], |row| row.get(0))
            .unwrap();
        assert_eq!(restored_state.0, Some(parent.document_id));
        assert_eq!(restored_state.1, 0);
        assert_eq!(restored_state.2, "active");
        assert_eq!(restored_state.3, None);
        assert_eq!(restored_state.4, None);
        assert_eq!(restored_state.5, restored_project_updated_at);
    }

    #[test]
    fn moving_an_already_trashed_document_is_idempotent() {
        let project = TestProject::create();
        let document = project.create_document("Capítulo");

        let first = project.trash(&document.document_id).unwrap();
        let first_project_updated_at: String = project
            .connection()
            .query_row("SELECT updated_at FROM project_info", [], |row| row.get(0))
            .unwrap();
        let second = project.trash(&document.document_id).unwrap();
        let second_project_updated_at: String = project
            .connection()
            .query_row("SELECT updated_at FROM project_info", [], |row| row.get(0))
            .unwrap();

        assert_eq!(second, first);
        assert_eq!(second_project_updated_at, first_project_updated_at);
    }

    #[test]
    fn reports_missing_documents_and_documents_outside_the_trash() {
        let project = TestProject::create();
        let document = project.create_document("Capítulo");

        let missing = project
            .trash("missing-document")
            .expect_err("a missing document must be reported");
        let active = project
            .restore(&document.document_id)
            .expect_err("an active document cannot be restored");

        assert!(matches!(missing, DocumentTrashError::DocumentNotFound));
        assert!(matches!(active, DocumentTrashError::DocumentNotTrashed));
    }

    #[test]
    fn rejects_blank_ids_before_accessing_the_project() {
        let trash_error = trash_document(TrashDocumentInput {
            project_path: PathBuf::from("missing.bookmaker"),
            document_id: "   ".to_owned(),
        })
        .expect_err("a blank trash ID must be rejected");
        let restore_error = restore_document(RestoreDocumentInput {
            project_path: PathBuf::from("missing.bookmaker"),
            document_id: "   ".to_owned(),
        })
        .expect_err("a blank restore ID must be rejected");

        assert!(matches!(trash_error, DocumentTrashError::InvalidDocumentId));
        assert!(matches!(
            restore_error,
            DocumentTrashError::InvalidDocumentId
        ));
    }

    #[test]
    fn rolls_back_trash_and_restore_when_the_project_timestamp_cannot_be_updated() {
        let project = TestProject::create();
        let document = project.create_document("Capítulo");
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

        let trash_error = project
            .trash(&document.document_id)
            .expect_err("project update failure must abort trash");
        assert!(matches!(
            trash_error,
            DocumentTrashError::TrashDatabase { .. }
        ));
        let active_status: String = project
            .connection()
            .query_row(
                "SELECT status FROM documents WHERE id = ?1",
                [&document.document_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(active_status, "active");

        let connection = project.connection();
        connection
            .execute("DROP TRIGGER reject_project_update", [])
            .unwrap();
        drop(connection);
        project.trash(&document.document_id).unwrap();
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

        let restore_error = project
            .restore(&document.document_id)
            .expect_err("project update failure must abort restore");
        assert!(matches!(
            restore_error,
            DocumentTrashError::RestoreDatabase { .. }
        ));
        let trashed_status: String = project
            .connection()
            .query_row(
                "SELECT status FROM documents WHERE id = ?1",
                [&document.document_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(trashed_status, "trashed");
    }
}
