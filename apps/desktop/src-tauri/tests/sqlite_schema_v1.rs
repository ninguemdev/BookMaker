use bookmaker_desktop_lib::persistence::migrations::SCHEMA_V1;
use rusqlite::{params, Connection, ErrorCode};

const EXPECTED_TABLES: [&str; 13] = [
    "assets",
    "contributor_roles",
    "contributors",
    "document_content",
    "documents",
    "export_profiles",
    "identifiers",
    "project_info",
    "project_metadata",
    "project_settings",
    "rights_holders",
    "schema_migrations",
    "style_profiles",
];

fn database_with_schema() -> Connection {
    let connection = Connection::open_in_memory().expect("in-memory SQLite must open");
    connection
        .pragma_update(None, "foreign_keys", true)
        .expect("foreign keys must be enabled");
    connection
        .execute_batch(SCHEMA_V1)
        .expect("schema v1 must be valid SQLite");
    connection
}

#[test]
fn creates_all_v1_tables_with_foreign_keys_enforced() {
    let connection = database_with_schema();

    let foreign_keys_enabled: bool = connection
        .pragma_query_value(None, "foreign_keys", |row| row.get(0))
        .expect("foreign key setting must be readable");
    assert!(foreign_keys_enabled);

    let mut statement = connection
        .prepare(
            "SELECT name FROM sqlite_schema \
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%' \
             ORDER BY name",
        )
        .expect("schema catalog query must prepare");
    let tables = statement
        .query_map([], |row| row.get::<_, String>(0))
        .expect("schema catalog query must execute")
        .collect::<rusqlite::Result<Vec<_>>>()
        .expect("table names must be readable");

    assert_eq!(tables, EXPECTED_TABLES);
}

#[test]
fn enforces_project_relationships() {
    let connection = database_with_schema();

    let error = connection
        .execute(
            "INSERT INTO project_metadata (
                project_id, title, language, rights_statement_mode
             ) VALUES (?1, ?2, ?3, ?4)",
            params!["missing-project", "Book", "pt-BR", "generated"],
        )
        .expect_err("metadata without a project must be rejected");

    assert_eq!(
        error.sqlite_error_code(),
        Some(ErrorCode::ConstraintViolation)
    );
}

#[test]
fn cascades_document_content_when_its_document_is_deleted() {
    let connection = database_with_schema();
    connection
        .execute(
            "INSERT INTO documents (
                id, parent_id, kind, role, title, position, status, created_at, updated_at
             ) VALUES (?1, NULL, ?2, ?3, ?4, 0, ?5, ?6, ?6)",
            params![
                "document-1",
                "flow",
                "chapter",
                "Chapter",
                "active",
                "2026-10-05T18:00:00Z"
            ],
        )
        .expect("document must be inserted");
    connection
        .execute(
            "INSERT INTO document_content (
                document_id, schema_version, json_content, updated_at
             ) VALUES (?1, 1, ?2, ?3)",
            params![
                "document-1",
                r#"{"type":"doc","content":[]}"#,
                "2026-10-05T18:00:00Z"
            ],
        )
        .expect("document content must be inserted");

    connection
        .execute("DELETE FROM documents WHERE id = ?1", ["document-1"])
        .expect("document must be deleted");

    let remaining_content: i64 = connection
        .query_row("SELECT COUNT(*) FROM document_content", [], |row| {
            row.get(0)
        })
        .expect("content count must be readable");
    assert_eq!(remaining_content, 0);
}

#[test]
fn rejects_invalid_json_and_duplicate_asset_paths() {
    let connection = database_with_schema();
    connection
        .execute(
            "INSERT INTO assets (
                id, type, relative_path, original_name, mime_type,
                byte_size, metadata_json, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                "asset-1",
                "image",
                "assets/images/cover.png",
                "cover.png",
                "image/png",
                100,
                "{}",
                "2026-10-05T18:00:00Z"
            ],
        )
        .expect("valid asset must be inserted");

    let duplicate_path_error = connection
        .execute(
            "INSERT INTO assets (
                id, type, relative_path, original_name, mime_type,
                metadata_json, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                "asset-2",
                "image",
                "assets/images/cover.png",
                "other-cover.png",
                "image/png",
                "{}",
                "2026-10-05T18:00:00Z"
            ],
        )
        .expect_err("duplicate asset paths must be rejected");
    assert_eq!(
        duplicate_path_error.sqlite_error_code(),
        Some(ErrorCode::ConstraintViolation)
    );

    let invalid_json_error = connection
        .execute(
            "INSERT INTO project_settings (key, value_json) VALUES (?1, ?2)",
            params!["invalid", "not-json"],
        )
        .expect_err("invalid settings JSON must be rejected");
    assert_eq!(
        invalid_json_error.sqlite_error_code(),
        Some(ErrorCode::ConstraintViolation)
    );
}
