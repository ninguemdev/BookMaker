use std::{error::Error, fmt};

use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};

#[derive(Debug, PartialEq, Eq)]
pub struct TrashedDocument {
    pub document_id: String,
    pub title: String,
    pub previous_status: String,
    pub trashed_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RestoredDocument {
    pub document_id: String,
    pub status: String,
}

#[derive(Debug)]
pub enum InternalTrashError {
    InvalidDocumentId,
    DocumentNotFound(String),
    DocumentNotTrashed(String),
    Database(rusqlite::Error),
}

impl fmt::Display for InternalTrashError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDocumentId => formatter.write_str("document ID cannot be blank"),
            Self::DocumentNotFound(document_id) => {
                write!(formatter, "document not found: {document_id}")
            }
            Self::DocumentNotTrashed(document_id) => {
                write!(
                    formatter,
                    "document is not in the internal trash: {document_id}"
                )
            }
            Self::Database(source) => write!(formatter, "internal trash database error: {source}"),
        }
    }
}

impl Error for InternalTrashError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(source) => Some(source),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for InternalTrashError {
    fn from(source: rusqlite::Error) -> Self {
        Self::Database(source)
    }
}

pub fn move_document_to_trash(
    connection: &mut Connection,
    document_id: &str,
) -> Result<TrashedDocument, InternalTrashError> {
    validate_document_id(document_id)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let status: Option<String> = transaction
        .query_row(
            "SELECT status FROM documents WHERE id = ?1",
            [document_id],
            |row| row.get(0),
        )
        .optional()?;
    let status =
        status.ok_or_else(|| InternalTrashError::DocumentNotFound(document_id.to_owned()))?;

    if status != "trashed" {
        transaction.execute(
            "UPDATE documents
             SET status_before_trash = status,
                 status = 'trashed',
                 trashed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?1",
            [document_id],
        )?;
    }

    let trashed = transaction.query_row(
        "SELECT id, title, status_before_trash, trashed_at
         FROM documents WHERE id = ?1",
        [document_id],
        |row| {
            Ok(TrashedDocument {
                document_id: row.get(0)?,
                title: row.get(1)?,
                previous_status: row.get(2)?,
                trashed_at: row.get(3)?,
            })
        },
    )?;
    transaction.commit()?;
    Ok(trashed)
}

pub fn restore_document_from_trash(
    connection: &mut Connection,
    document_id: &str,
) -> Result<RestoredDocument, InternalTrashError> {
    validate_document_id(document_id)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let state: Option<(String, Option<String>)> = transaction
        .query_row(
            "SELECT status, status_before_trash FROM documents WHERE id = ?1",
            [document_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let (status, previous_status) =
        state.ok_or_else(|| InternalTrashError::DocumentNotFound(document_id.to_owned()))?;
    if status != "trashed" {
        return Err(InternalTrashError::DocumentNotTrashed(
            document_id.to_owned(),
        ));
    }

    let restored_status = previous_status.unwrap_or_else(|| "active".to_owned());
    transaction.execute(
        "UPDATE documents
         SET status = ?2,
             status_before_trash = NULL,
             trashed_at = NULL,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?1",
        params![document_id, restored_status],
    )?;
    transaction.commit()?;

    Ok(RestoredDocument {
        document_id: document_id.to_owned(),
        status: restored_status,
    })
}

pub fn list_trashed_documents(
    connection: &Connection,
) -> Result<Vec<TrashedDocument>, InternalTrashError> {
    let mut statement = connection.prepare(
        "SELECT id, title, status_before_trash, trashed_at
         FROM documents
         WHERE status = 'trashed'
         ORDER BY trashed_at DESC, id",
    )?;
    let documents = statement
        .query_map([], |row| {
            Ok(TrashedDocument {
                document_id: row.get(0)?,
                title: row.get(1)?,
                previous_status: row.get(2)?,
                trashed_at: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(documents)
}

fn validate_document_id(document_id: &str) -> Result<(), InternalTrashError> {
    if document_id.trim().is_empty() {
        return Err(InternalTrashError::InvalidDocumentId);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::migrations::run_migrations;

    fn database_with_document() -> Connection {
        let mut connection = Connection::open_in_memory().expect("database must open");
        run_migrations(&mut connection).expect("database must migrate");
        connection
            .execute(
                "INSERT INTO documents (
                    id, parent_id, kind, role, title, position, status, created_at, updated_at
                 ) VALUES (
                    'document-1', NULL, 'flow', 'chapter', 'Chapter', 0,
                    'draft', '2026-10-05T18:00:00Z', '2026-10-05T18:00:00Z'
                 )",
                [],
            )
            .expect("document must be inserted");
        connection
            .execute(
                "INSERT INTO document_content (
                    document_id, schema_version, json_content, updated_at
                 ) VALUES (
                    'document-1', 1, '{\"type\":\"doc\"}', '2026-10-05T18:00:00Z'
                 )",
                [],
            )
            .expect("document content must be inserted");
        connection
    }

    #[test]
    fn moves_a_document_to_trash_without_deleting_its_content() {
        let mut connection = database_with_document();

        let trashed = move_document_to_trash(&mut connection, "document-1").unwrap();

        assert_eq!(trashed.document_id, "document-1");
        assert_eq!(trashed.previous_status, "draft");
        assert!(!trashed.trashed_at.is_empty());
        let content_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM document_content", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(content_count, 1);
        assert_eq!(list_trashed_documents(&connection).unwrap(), vec![trashed]);
    }

    #[test]
    fn moving_an_already_trashed_document_is_idempotent() {
        let mut connection = database_with_document();
        let first = move_document_to_trash(&mut connection, "document-1").unwrap();

        let second = move_document_to_trash(&mut connection, "document-1").unwrap();

        assert_eq!(second, first);
    }

    #[test]
    fn restores_the_status_that_preceded_the_trash() {
        let mut connection = database_with_document();
        move_document_to_trash(&mut connection, "document-1").unwrap();

        let restored = restore_document_from_trash(&mut connection, "document-1").unwrap();

        assert_eq!(
            restored,
            RestoredDocument {
                document_id: "document-1".to_owned(),
                status: "draft".to_owned(),
            }
        );
        assert!(list_trashed_documents(&connection).unwrap().is_empty());
        let trash_metadata: (Option<String>, Option<String>) = connection
            .query_row(
                "SELECT trashed_at, status_before_trash FROM documents WHERE id = 'document-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(trash_metadata, (None, None));
    }

    #[test]
    fn refuses_to_restore_a_document_that_is_not_trashed() {
        let mut connection = database_with_document();

        let error = restore_document_from_trash(&mut connection, "document-1")
            .expect_err("active document must not be restored");

        assert!(matches!(error, InternalTrashError::DocumentNotTrashed(_)));
    }
}
