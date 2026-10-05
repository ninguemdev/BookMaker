use std::{error::Error, fmt, fmt::Write as _};

use rusqlite::{params, Connection, TransactionBehavior};
use sha2::{Digest, Sha256};

pub const CURRENT_SCHEMA_VERSION: i64 = 2;
pub const SCHEMA_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../packages/project-format/migrations/0001_initial.sql"
));
pub const SCHEMA_V2: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../packages/project-format/migrations/0002_internal_trash.sql"
));

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial",
        sql: SCHEMA_V1,
    },
    Migration {
        version: CURRENT_SCHEMA_VERSION,
        name: "internal_trash",
        sql: SCHEMA_V2,
    },
];

#[derive(Debug, PartialEq, Eq)]
pub struct MigrationReport {
    pub previous_version: i64,
    pub current_version: i64,
    pub applied_versions: Vec<i64>,
}

#[derive(Debug)]
pub enum MigrationError {
    Database(rusqlite::Error),
    ForeignKeysDisabled,
    InvalidDefinition(String),
    UnsupportedDatabaseVersion {
        database_version: i64,
        supported_version: i64,
    },
    HistoryMismatch {
        version: i64,
        details: String,
    },
    ApplyFailed {
        version: i64,
        name: &'static str,
        source: rusqlite::Error,
    },
}

impl fmt::Display for MigrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(source) => write!(formatter, "failed to inspect migration state: {source}"),
            Self::ForeignKeysDisabled => {
                formatter.write_str("SQLite foreign key enforcement could not be enabled")
            }
            Self::InvalidDefinition(details) => {
                write!(formatter, "invalid migration definition: {details}")
            }
            Self::UnsupportedDatabaseVersion {
                database_version,
                supported_version,
            } => write!(
                formatter,
                "project database version {database_version} is newer than supported version {supported_version}"
            ),
            Self::HistoryMismatch { version, details } => {
                write!(formatter, "migration history mismatch at version {version}: {details}")
            }
            Self::ApplyFailed {
                version,
                name,
                source,
            } => write!(
                formatter,
                "failed to apply migration {version} ({name}): {source}"
            ),
        }
    }
}

impl Error for MigrationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(source) | Self::ApplyFailed { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for MigrationError {
    fn from(source: rusqlite::Error) -> Self {
        Self::Database(source)
    }
}

pub fn run_migrations(connection: &mut Connection) -> Result<MigrationReport, MigrationError> {
    run_migrations_with(connection, MIGRATIONS)
}

#[derive(Clone, Copy)]
struct Migration {
    version: i64,
    name: &'static str,
    sql: &'static str,
}

#[derive(Debug)]
struct AppliedMigration {
    version: i64,
    name: String,
    checksum: Option<String>,
}

fn run_migrations_with(
    connection: &mut Connection,
    migrations: &[Migration],
) -> Result<MigrationReport, MigrationError> {
    validate_definitions(migrations)?;
    enable_foreign_keys(connection)?;

    let applied = load_applied_migrations(connection)?;
    validate_history(&applied, migrations)?;

    let previous_version = applied.last().map_or(0, |migration| migration.version);
    let mut applied_versions = Vec::new();

    for migration in &migrations[applied.len()..] {
        apply_migration(connection, migration)?;
        applied_versions.push(migration.version);
    }

    Ok(MigrationReport {
        previous_version,
        current_version: migrations.last().map_or(0, |migration| migration.version),
        applied_versions,
    })
}

fn validate_definitions(migrations: &[Migration]) -> Result<(), MigrationError> {
    for (index, migration) in migrations.iter().enumerate() {
        let expected_version = index as i64 + 1;
        if migration.version != expected_version {
            return Err(MigrationError::InvalidDefinition(format!(
                "expected version {expected_version}, found {}",
                migration.version
            )));
        }

        if migration.name.trim().is_empty() || migration.sql.trim().is_empty() {
            return Err(MigrationError::InvalidDefinition(format!(
                "migration {} must have a name and SQL",
                migration.version
            )));
        }
    }

    Ok(())
}

fn enable_foreign_keys(connection: &Connection) -> Result<(), MigrationError> {
    connection.pragma_update(None, "foreign_keys", true)?;
    let enabled =
        connection.pragma_query_value(None, "foreign_keys", |row| row.get::<_, bool>(0))?;

    if !enabled {
        return Err(MigrationError::ForeignKeysDisabled);
    }

    Ok(())
}

fn load_applied_migrations(
    connection: &Connection,
) -> Result<Vec<AppliedMigration>, MigrationError> {
    let history_exists = connection.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM sqlite_schema
            WHERE type = 'table' AND name = 'schema_migrations'
         )",
        [],
        |row| row.get::<_, bool>(0),
    )?;

    if !history_exists {
        return Ok(Vec::new());
    }

    let mut statement = connection.prepare(
        "SELECT version, name, checksum
         FROM schema_migrations
         ORDER BY version",
    )?;
    let migrations = statement
        .query_map([], |row| {
            Ok(AppliedMigration {
                version: row.get(0)?,
                name: row.get(1)?,
                checksum: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(migrations)
}

fn validate_history(
    applied: &[AppliedMigration],
    migrations: &[Migration],
) -> Result<(), MigrationError> {
    let supported_version = migrations.last().map_or(0, |migration| migration.version);

    if let Some(database_version) = applied.last().map(|migration| migration.version) {
        if database_version > supported_version {
            return Err(MigrationError::UnsupportedDatabaseVersion {
                database_version,
                supported_version,
            });
        }
    }

    for (index, recorded) in applied.iter().enumerate() {
        let Some(expected) = migrations.get(index) else {
            return Err(MigrationError::UnsupportedDatabaseVersion {
                database_version: recorded.version,
                supported_version,
            });
        };

        if recorded.version != expected.version {
            return Err(MigrationError::HistoryMismatch {
                version: recorded.version,
                details: format!("expected migration version {}", expected.version),
            });
        }

        let expected_checksum = migration_checksum(expected.sql);
        if recorded.name != expected.name
            || recorded.checksum.as_deref() != Some(expected_checksum.as_str())
        {
            return Err(MigrationError::HistoryMismatch {
                version: recorded.version,
                details: "recorded name or checksum differs from the application migration"
                    .to_owned(),
            });
        }
    }

    Ok(())
}

fn apply_migration(
    connection: &mut Connection,
    migration: &Migration,
) -> Result<(), MigrationError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(MigrationError::Database)?;

    transaction
        .execute_batch(migration.sql)
        .map_err(|source| MigrationError::ApplyFailed {
            version: migration.version,
            name: migration.name,
            source,
        })?;

    transaction
        .execute(
            "INSERT INTO schema_migrations (
                version, name, applied_at, checksum
             ) VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), ?3)",
            params![
                migration.version,
                migration.name,
                migration_checksum(migration.sql)
            ],
        )
        .map_err(|source| MigrationError::ApplyFailed {
            version: migration.version,
            name: migration.name,
            source,
        })?;

    transaction
        .commit()
        .map_err(|source| MigrationError::ApplyFailed {
            version: migration.version,
            name: migration.name,
            source,
        })
}

fn migration_checksum(sql: &str) -> String {
    let canonical_sql = sql.replace("\r\n", "\n");
    let digest = Sha256::digest(canonical_sql.as_bytes());
    let mut checksum = String::with_capacity(digest.len() * 2);

    for byte in digest {
        write!(checksum, "{byte:02x}").expect("writing to a String cannot fail");
    }

    checksum
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::OptionalExtension;

    const HISTORY_TABLE: &str = "
        CREATE TABLE schema_migrations (
          version INTEGER PRIMARY KEY NOT NULL,
          name TEXT NOT NULL,
          applied_at TEXT NOT NULL,
          checksum TEXT
        ) STRICT;
    ";

    #[test]
    fn migrates_a_new_database_and_reports_the_applied_version() {
        let mut connection = Connection::open_in_memory().expect("database must open");

        let report = run_migrations(&mut connection).expect("migration must succeed");

        assert_eq!(
            report,
            MigrationReport {
                previous_version: 0,
                current_version: CURRENT_SCHEMA_VERSION,
                applied_versions: vec![1, 2],
            }
        );
        let recorded: (i64, String, String) = connection
            .query_row(
                "SELECT version, name, checksum FROM schema_migrations",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("migration history must be recorded");
        assert_eq!(recorded.0, 1);
        assert_eq!(recorded.1, "initial");
        assert_eq!(recorded.2, migration_checksum(SCHEMA_V1));
    }

    #[test]
    fn does_not_reapply_an_existing_migration() {
        let mut connection = Connection::open_in_memory().expect("database must open");
        run_migrations(&mut connection).expect("first migration run must succeed");

        let report = run_migrations(&mut connection).expect("second migration run must succeed");

        assert_eq!(report.previous_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(report.current_version, CURRENT_SCHEMA_VERSION);
        assert!(report.applied_versions.is_empty());
    }

    #[test]
    fn rejects_a_changed_applied_migration() {
        let mut connection = Connection::open_in_memory().expect("database must open");
        run_migrations(&mut connection).expect("migration must succeed");
        connection
            .execute(
                "UPDATE schema_migrations SET checksum = 'changed' WHERE version = 1",
                [],
            )
            .expect("test history must be changed");

        let error = run_migrations(&mut connection).expect_err("changed history must fail");

        assert!(matches!(
            error,
            MigrationError::HistoryMismatch { version: 1, .. }
        ));
    }

    #[test]
    fn rejects_a_database_newer_than_the_application() {
        let mut connection = Connection::open_in_memory().expect("database must open");
        run_migrations(&mut connection).expect("migration must succeed");
        connection
            .execute(
                "INSERT INTO schema_migrations (
                    version, name, applied_at, checksum
                 ) VALUES (3, 'future', '2026-10-05T18:00:00Z', 'future')",
                [],
            )
            .expect("future migration record must be inserted");

        let error = run_migrations(&mut connection).expect_err("newer database must fail");

        assert!(matches!(
            error,
            MigrationError::UnsupportedDatabaseVersion {
                database_version: 3,
                supported_version: CURRENT_SCHEMA_VERSION,
            }
        ));
    }

    #[test]
    fn rolls_back_only_the_failed_migration() {
        let migrations = [
            Migration {
                version: 1,
                name: "base",
                sql: concat!(
                    "CREATE TABLE preserved (id INTEGER PRIMARY KEY) STRICT;",
                    "CREATE TABLE schema_migrations (",
                    "version INTEGER PRIMARY KEY NOT NULL,",
                    "name TEXT NOT NULL,",
                    "applied_at TEXT NOT NULL,",
                    "checksum TEXT",
                    ") STRICT;"
                ),
            },
            Migration {
                version: 2,
                name: "broken",
                sql: "CREATE TABLE partial (id INTEGER); INVALID SQL;",
            },
        ];
        let mut connection = Connection::open_in_memory().expect("database must open");

        let error = run_migrations_with(&mut connection, &migrations)
            .expect_err("invalid migration must fail");

        assert!(matches!(
            error,
            MigrationError::ApplyFailed {
                version: 2,
                name: "broken",
                ..
            }
        ));
        let preserved_exists = table_exists(&connection, "preserved");
        let partial_exists = table_exists(&connection, "partial");
        let recorded_versions: i64 = connection
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("history count must be readable");

        assert!(preserved_exists);
        assert!(!partial_exists);
        assert_eq!(recorded_versions, 1);
    }

    #[test]
    fn rejects_non_sequential_migration_definitions() {
        let migrations = [Migration {
            version: 2,
            name: "invalid-start",
            sql: HISTORY_TABLE,
        }];
        let mut connection = Connection::open_in_memory().expect("database must open");

        let error = run_migrations_with(&mut connection, &migrations)
            .expect_err("invalid definitions must fail");

        assert!(matches!(error, MigrationError::InvalidDefinition(_)));
    }

    #[test]
    fn checksum_is_stable_across_platform_line_endings() {
        assert_eq!(
            migration_checksum("CREATE TABLE example (id INTEGER);\n"),
            migration_checksum("CREATE TABLE example (id INTEGER);\r\n")
        );
    }

    #[test]
    fn migrates_a_v1_database_without_losing_documents() {
        let mut connection = Connection::open_in_memory().expect("database must open");
        run_migrations_with(&mut connection, &MIGRATIONS[..1]).expect("v1 fixture must be created");
        connection
            .execute(
                "INSERT INTO documents (
                    id, parent_id, kind, role, title, position, status, created_at, updated_at
                 ) VALUES (
                    'document-1', NULL, 'flow', 'chapter', 'Preserved', 0,
                    'draft', '2026-10-05T18:00:00Z', '2026-10-05T18:00:00Z'
                 )",
                [],
            )
            .expect("v1 fixture document must be inserted");

        let report = run_migrations(&mut connection).expect("v2 migration must succeed");

        assert_eq!(report.previous_version, 1);
        assert_eq!(report.applied_versions, vec![2]);
        let document: (String, Option<String>, Option<String>) = connection
            .query_row(
                "SELECT title, trashed_at, status_before_trash
                 FROM documents WHERE id = 'document-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("document must survive the migration");
        assert_eq!(document, ("Preserved".to_owned(), None, None));
    }

    fn table_exists(connection: &Connection, table: &str) -> bool {
        connection
            .query_row(
                "SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?1",
                [table],
                |_| Ok(()),
            )
            .optional()
            .expect("schema query must succeed")
            .is_some()
    }
}
