use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use rusqlite::{params, Connection, OpenFlags, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};

use crate::{
    application::open_project::{open_project, OpenProjectError, OpenProjectInput},
    persistence::project_manifest::DATABASE_FILE,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveDocumentInput {
    pub project_path: PathBuf,
    pub document_id: String,
    pub new_parent_id: Option<String>,
    pub position: i64,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovedDocument {
    pub document_id: String,
    pub parent_id: Option<String>,
    pub position: i64,
    pub updated_at: String,
}

#[derive(Debug)]
pub enum MoveDocumentError {
    InvalidDocumentId,
    InvalidParentId,
    InvalidPosition,
    DocumentNotFound,
    ParentNotFound,
    InvalidHierarchy,
    OpenProject(OpenProjectError),
    Database {
        path: PathBuf,
        source: rusqlite::Error,
    },
}

impl MoveDocumentError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidDocumentId => "document.invalid_id",
            Self::InvalidParentId => "document.invalid_parent_id",
            Self::InvalidPosition => "document.invalid_position",
            Self::DocumentNotFound => "document.not_found",
            Self::ParentNotFound => "document.parent_not_found",
            Self::InvalidHierarchy => "document.invalid_hierarchy",
            Self::OpenProject(source) => source.code(),
            Self::Database { .. } => "document.move_failed",
        }
    }

    pub fn user_message(&self) -> &'static str {
        match self {
            Self::InvalidDocumentId | Self::DocumentNotFound => "O documento não foi encontrado.",
            Self::InvalidParentId | Self::ParentNotFound => {
                "O destino do documento não foi encontrado."
            }
            Self::InvalidPosition => "A posição escolhida para o documento é inválida.",
            Self::InvalidHierarchy => "O documento não pode ser movido para dentro de si mesmo.",
            Self::OpenProject(source) => source.user_message(),
            Self::Database { .. } => "Não foi possível mover o documento.",
        }
    }
}

impl fmt::Display for MoveDocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDocumentId => formatter.write_str("document ID cannot be blank"),
            Self::InvalidParentId => formatter.write_str("parent document ID cannot be blank"),
            Self::InvalidPosition => formatter.write_str("document position is out of bounds"),
            Self::DocumentNotFound => formatter.write_str("document was not found"),
            Self::ParentNotFound => formatter.write_str("parent document was not found"),
            Self::InvalidHierarchy => {
                formatter.write_str("document move would create a hierarchy cycle")
            }
            Self::OpenProject(source) => write!(formatter, "cannot open project: {source}"),
            Self::Database { path, source } => write!(
                formatter,
                "document database operation failed at {}: {source}",
                path.display()
            ),
        }
    }
}

impl Error for MoveDocumentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OpenProject(source) => Some(source),
            Self::Database { source, .. } => Some(source),
            Self::InvalidDocumentId
            | Self::InvalidParentId
            | Self::InvalidPosition
            | Self::DocumentNotFound
            | Self::ParentNotFound
            | Self::InvalidHierarchy => None,
        }
    }
}

pub fn move_document(input: MoveDocumentInput) -> Result<MovedDocument, MoveDocumentError> {
    let document_id = validate_id(&input.document_id, MoveDocumentError::InvalidDocumentId)?;
    let new_parent_id = input
        .new_parent_id
        .as_deref()
        .map(|parent_id| validate_id(parent_id, MoveDocumentError::InvalidParentId))
        .transpose()?;
    if input.position < 0 {
        return Err(MoveDocumentError::InvalidPosition);
    }

    let project = open_project(OpenProjectInput {
        path: input.project_path,
    })
    .map_err(MoveDocumentError::OpenProject)?;
    let database_path = project.path.join(DATABASE_FILE);
    let mut connection =
        Connection::open_with_flags(&database_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
            .map_err(|source| database_error(&database_path, source))?;
    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(|source| database_error(&database_path, source))?;

    move_document_in_tree(
        &mut connection,
        &database_path,
        document_id,
        new_parent_id,
        input.position,
    )
}

fn validate_id(id: &str, error: MoveDocumentError) -> Result<String, MoveDocumentError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(error);
    }
    Ok(id.to_owned())
}

fn move_document_in_tree(
    connection: &mut Connection,
    database_path: &Path,
    document_id: String,
    new_parent_id: Option<String>,
    new_position: i64,
) -> Result<MovedDocument, MoveDocumentError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|source| database_error(database_path, source))?;
    let current: Option<(Option<String>, i64)> = transaction
        .query_row(
            "SELECT parent_id, position FROM documents WHERE id = ?1",
            [&document_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|source| database_error(database_path, source))?;
    let (current_parent_id, current_position) =
        current.ok_or(MoveDocumentError::DocumentNotFound)?;

    if let Some(parent_id) = &new_parent_id {
        let parent_exists = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM documents WHERE id = ?1)",
                [parent_id],
                |row| row.get::<_, bool>(0),
            )
            .map_err(|source| database_error(database_path, source))?;
        if !parent_exists {
            return Err(MoveDocumentError::ParentNotFound);
        }

        let creates_cycle = transaction
            .query_row(
                "WITH RECURSIVE ancestors(id, parent_id) AS (
                    SELECT id, parent_id FROM documents WHERE id = ?1
                    UNION
                    SELECT documents.id, documents.parent_id
                    FROM documents
                    JOIN ancestors ON documents.id = ancestors.parent_id
                 )
                 SELECT EXISTS(SELECT 1 FROM ancestors WHERE id = ?2)",
                params![parent_id, document_id],
                |row| row.get::<_, bool>(0),
            )
            .map_err(|source| database_error(database_path, source))?;
        if creates_cycle {
            return Err(MoveDocumentError::InvalidHierarchy);
        }
    }

    let target_sibling_count = transaction
        .query_row(
            "SELECT COUNT(*)
             FROM documents
             WHERE parent_id IS ?1 AND id <> ?2",
            params![new_parent_id, document_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|source| database_error(database_path, source))?;
    if new_position > target_sibling_count {
        return Err(MoveDocumentError::InvalidPosition);
    }

    let updated_at = transaction
        .query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ', 'now')", [], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|source| database_error(database_path, source))?;
    transaction
        .execute(
            "UPDATE documents
             SET position = position - 1, updated_at = ?3
             WHERE parent_id IS ?1 AND id <> ?2 AND position > ?4",
            params![current_parent_id, document_id, updated_at, current_position],
        )
        .map_err(|source| database_error(database_path, source))?;
    transaction
        .execute(
            "UPDATE documents
             SET position = position + 1, updated_at = ?3
             WHERE parent_id IS ?1 AND id <> ?2 AND position >= ?4",
            params![new_parent_id, document_id, updated_at, new_position],
        )
        .map_err(|source| database_error(database_path, source))?;
    transaction
        .execute(
            "UPDATE documents
             SET parent_id = ?2, position = ?3, updated_at = ?4
             WHERE id = ?1",
            params![document_id, new_parent_id, new_position, updated_at],
        )
        .map_err(|source| database_error(database_path, source))?;
    transaction
        .execute("UPDATE project_info SET updated_at = ?1", [&updated_at])
        .map_err(|source| database_error(database_path, source))?;
    transaction
        .commit()
        .map_err(|source| database_error(database_path, source))?;

    Ok(MovedDocument {
        document_id,
        parent_id: new_parent_id,
        position: new_position,
        updated_at,
    })
}

fn database_error(path: &Path, source: rusqlite::Error) -> MoveDocumentError {
    MoveDocumentError::Database {
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

    #[derive(Debug, PartialEq, Eq)]
    struct TreeEntry {
        id: String,
        parent_id: Option<String>,
        position: i64,
        updated_at: String,
    }

    struct TestProject {
        parent: PathBuf,
        path: PathBuf,
    }

    impl TestProject {
        fn create() -> Self {
            let parent = std::env::temp_dir()
                .join(format!("bookmaker-move-document-test-{}", Uuid::now_v7()));
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
            })
            .expect("test document must be created")
        }

        fn connection(&self) -> Connection {
            Connection::open(self.path.join(DATABASE_FILE)).expect("test database must open")
        }

        fn children(&self, parent_id: Option<&str>) -> Vec<TreeEntry> {
            let connection = self.connection();
            let mut statement = connection
                .prepare(
                    "SELECT id, parent_id, position, updated_at
                     FROM documents
                     WHERE parent_id IS ?1
                     ORDER BY position, id",
                )
                .unwrap();
            statement
                .query_map([parent_id], |row| {
                    Ok(TreeEntry {
                        id: row.get(0)?,
                        parent_id: row.get(1)?,
                        position: row.get(2)?,
                        updated_at: row.get(3)?,
                    })
                })
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap()
        }

        fn move_document(
            &self,
            document_id: &str,
            new_parent_id: Option<&str>,
            position: i64,
        ) -> Result<MovedDocument, MoveDocumentError> {
            move_document(MoveDocumentInput {
                project_path: self.path.clone(),
                document_id: document_id.to_owned(),
                new_parent_id: new_parent_id.map(str::to_owned),
                position,
            })
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
    fn reorders_root_documents_and_preserves_content_timestamps() {
        let project = TestProject::create();
        let first = project.create_document("Primeiro");
        let second = project.create_document("Segundo");
        let third = project.create_document("Terceiro");
        let connection = project.connection();
        connection
            .execute(
                "UPDATE documents SET updated_at = '2000-01-01T00:00:00.000Z'",
                [],
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
                [&first.document_id],
                |row| row.get(0),
            )
            .unwrap();
        drop(connection);

        let moved = project.move_document(&first.document_id, None, 2).unwrap();

        assert_eq!(moved.document_id, first.document_id);
        assert_eq!(moved.parent_id, None);
        assert_eq!(moved.position, 2);
        let roots = project.children(None);
        assert_eq!(
            roots
                .iter()
                .map(|entry| (&entry.id, entry.position))
                .collect::<Vec<_>>(),
            vec![
                (&second.document_id, 0),
                (&third.document_id, 1),
                (&first.document_id, 2)
            ]
        );
        assert!(roots
            .iter()
            .all(|entry| entry.updated_at == moved.updated_at));
        let connection = project.connection();
        let project_updated_at: String = connection
            .query_row("SELECT updated_at FROM project_info", [], |row| row.get(0))
            .unwrap();
        let persisted_content_updated_at: String = connection
            .query_row(
                "SELECT updated_at FROM document_content WHERE document_id = ?1",
                [&first.document_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(project_updated_at, moved.updated_at);
        assert_eq!(persisted_content_updated_at, content_updated_at);
    }

    #[test]
    fn moves_documents_between_parents_and_inserts_at_the_requested_position() {
        let project = TestProject::create();
        let parent = project.create_document("Parte");
        let first_child = project.create_document("Capítulo 1");
        let second_child = project.create_document("Capítulo 2");
        let remaining_root = project.create_document("Apêndice");

        project
            .move_document(&first_child.document_id, Some(&parent.document_id), 0)
            .unwrap();
        project
            .move_document(&second_child.document_id, Some(&parent.document_id), 0)
            .unwrap();

        let roots = project.children(None);
        assert_eq!(
            roots
                .iter()
                .map(|entry| (&entry.id, entry.position))
                .collect::<Vec<_>>(),
            vec![(&parent.document_id, 0), (&remaining_root.document_id, 1)]
        );
        let children = project.children(Some(&parent.document_id));
        assert_eq!(
            children
                .iter()
                .map(|entry| (&entry.id, entry.position))
                .collect::<Vec<_>>(),
            vec![
                (&second_child.document_id, 0),
                (&first_child.document_id, 1)
            ]
        );
    }

    #[test]
    fn rejects_moves_that_would_create_a_cycle() {
        let project = TestProject::create();
        let parent = project.create_document("Parte");
        let child = project.create_document("Capítulo");
        let grandchild = project.create_document("Seção");
        project
            .move_document(&child.document_id, Some(&parent.document_id), 0)
            .unwrap();
        project
            .move_document(&grandchild.document_id, Some(&child.document_id), 0)
            .unwrap();

        let self_move = project
            .move_document(&parent.document_id, Some(&parent.document_id), 0)
            .expect_err("a document cannot be its own parent");
        let descendant_move = project
            .move_document(&parent.document_id, Some(&grandchild.document_id), 0)
            .expect_err("a document cannot move below its descendant");

        assert!(matches!(self_move, MoveDocumentError::InvalidHierarchy));
        assert!(matches!(
            descendant_move,
            MoveDocumentError::InvalidHierarchy
        ));
        assert_eq!(project.children(None)[0].id, parent.document_id);
    }

    #[test]
    fn rejects_missing_documents_and_parents() {
        let project = TestProject::create();
        let document = project.create_document("Capítulo");

        let missing_document = project
            .move_document("missing-document", None, 0)
            .expect_err("a missing document must be reported");
        let missing_parent = project
            .move_document(&document.document_id, Some("missing-parent"), 0)
            .expect_err("a missing parent must be reported");

        assert!(matches!(
            missing_document,
            MoveDocumentError::DocumentNotFound
        ));
        assert!(matches!(missing_parent, MoveDocumentError::ParentNotFound));
    }

    #[test]
    fn rejects_invalid_input_before_accessing_the_project() {
        let invalid_document_id = move_document(MoveDocumentInput {
            project_path: PathBuf::from("missing.bookmaker"),
            document_id: "   ".to_owned(),
            new_parent_id: None,
            position: 0,
        })
        .expect_err("blank document IDs must be rejected");
        let invalid_parent_id = move_document(MoveDocumentInput {
            project_path: PathBuf::from("missing.bookmaker"),
            document_id: "document-1".to_owned(),
            new_parent_id: Some("   ".to_owned()),
            position: 0,
        })
        .expect_err("blank parent IDs must be rejected");
        let invalid_position = move_document(MoveDocumentInput {
            project_path: PathBuf::from("missing.bookmaker"),
            document_id: "document-1".to_owned(),
            new_parent_id: None,
            position: -1,
        })
        .expect_err("negative positions must be rejected");

        assert!(matches!(
            invalid_document_id,
            MoveDocumentError::InvalidDocumentId
        ));
        assert!(matches!(
            invalid_parent_id,
            MoveDocumentError::InvalidParentId
        ));
        assert!(matches!(
            invalid_position,
            MoveDocumentError::InvalidPosition
        ));
    }

    #[test]
    fn rejects_a_position_beyond_the_destination_without_changing_the_tree() {
        let project = TestProject::create();
        let first = project.create_document("Primeiro");
        let second = project.create_document("Segundo");
        let before = project.children(None);

        let error = project
            .move_document(&first.document_id, None, 2)
            .expect_err("a final position beyond the sibling count must fail");

        assert!(matches!(error, MoveDocumentError::InvalidPosition));
        assert_eq!(project.children(None), before);
        assert_eq!(project.children(None)[1].id, second.document_id);
    }

    #[test]
    fn rolls_back_all_positions_when_the_project_timestamp_cannot_be_updated() {
        let project = TestProject::create();
        let first = project.create_document("Primeiro");
        project.create_document("Segundo");
        project.create_document("Terceiro");
        let before = project.children(None);
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

        let error = project
            .move_document(&first.document_id, None, 2)
            .expect_err("project update failure must abort the move");

        assert!(matches!(error, MoveDocumentError::Database { .. }));
        assert_eq!(project.children(None), before);
    }
}
