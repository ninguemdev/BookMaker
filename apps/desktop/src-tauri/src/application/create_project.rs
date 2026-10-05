use std::{
    error::Error,
    ffi::OsStr,
    fmt, fs,
    fs::OpenOptions,
    io::{self, Write},
    path::{Path, PathBuf},
};

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::persistence::migrations::{run_migrations, MigrationError};
use crate::persistence::project_manifest::{
    ProjectManifest, DATABASE_FILE, MANIFEST_FILE, PROJECT_EXTENSION, PROJECT_FORMAT_VERSION,
};
#[cfg(test)]
use crate::persistence::project_manifest::{PROJECT_CREATOR, PROJECT_FORMAT};

const REQUIRED_DIRECTORIES: &[&str] = &[
    "assets/images",
    "assets/covers",
    "assets/ornaments",
    "assets/fonts",
    "recovery",
    "snapshots",
    "cache",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectInput {
    pub destination: PathBuf,
    pub title: String,
    pub language: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedProject {
    pub project_id: String,
    pub path: PathBuf,
}

#[derive(Debug)]
pub enum CreateProjectError {
    InvalidDestination(&'static str),
    InvalidTitle,
    InvalidLanguage,
    DestinationExists(PathBuf),
    Io {
        operation: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    Database {
        path: PathBuf,
        source: rusqlite::Error,
    },
    Migration(MigrationError),
    ManifestSerialization(serde_json::Error),
    RollbackFailed {
        destination: PathBuf,
        original: Box<CreateProjectError>,
        source: io::Error,
    },
}

impl CreateProjectError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidDestination(_) => "project.invalid_destination",
            Self::InvalidTitle => "project.invalid_title",
            Self::InvalidLanguage => "project.invalid_language",
            Self::DestinationExists(_) => "project.destination_exists",
            Self::RollbackFailed { .. } => "project.rollback_failed",
            Self::Io { .. }
            | Self::Database { .. }
            | Self::Migration(_)
            | Self::ManifestSerialization(_) => "project.create_failed",
        }
    }

    pub fn user_message(&self) -> &'static str {
        match self {
            Self::InvalidDestination(_) => "Escolha uma pasta de projeto válida.",
            Self::InvalidTitle => "Informe um título para o projeto.",
            Self::InvalidLanguage => "Informe o idioma do livro.",
            Self::DestinationExists(_) => "Já existe um arquivo ou pasta nesse destino.",
            Self::RollbackFailed { .. } => {
                "A criação falhou e a pasta incompleta não pôde ser removida."
            }
            Self::Io { .. }
            | Self::Database { .. }
            | Self::Migration(_)
            | Self::ManifestSerialization(_) => "Não foi possível criar o projeto.",
        }
    }
}

impl fmt::Display for CreateProjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDestination(reason) => {
                write!(formatter, "invalid project destination: {reason}")
            }
            Self::InvalidTitle => formatter.write_str("project title cannot be blank"),
            Self::InvalidLanguage => formatter.write_str("project language cannot be blank"),
            Self::DestinationExists(path) => {
                write!(
                    formatter,
                    "project destination already exists: {}",
                    path.display()
                )
            }
            Self::Io {
                operation,
                path,
                source,
            } => write!(
                formatter,
                "failed to {operation} at {}: {source}",
                path.display()
            ),
            Self::Database { path, source } => {
                write!(
                    formatter,
                    "database operation failed at {}: {source}",
                    path.display()
                )
            }
            Self::Migration(source) => write!(formatter, "project migration failed: {source}"),
            Self::ManifestSerialization(source) => {
                write!(formatter, "manifest serialization failed: {source}")
            }
            Self::RollbackFailed {
                destination,
                original,
                source,
            } => write!(
                formatter,
                "{original}; cleanup of {} also failed: {source}",
                destination.display()
            ),
        }
    }
}

impl Error for CreateProjectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } | Self::RollbackFailed { source, .. } => Some(source),
            Self::Database { source, .. } => Some(source),
            Self::Migration(source) => Some(source),
            Self::ManifestSerialization(source) => Some(source),
            Self::InvalidDestination(_)
            | Self::InvalidTitle
            | Self::InvalidLanguage
            | Self::DestinationExists(_) => None,
        }
    }
}

#[derive(Debug)]
struct ValidatedCreateProject {
    destination: PathBuf,
    title: String,
    language: String,
}

pub fn create_project(input: CreateProjectInput) -> Result<CreatedProject, CreateProjectError> {
    let project_id = Uuid::now_v7().to_string();
    create_project_with_id(input, project_id)
}

fn create_project_with_id(
    input: CreateProjectInput,
    project_id: String,
) -> Result<CreatedProject, CreateProjectError> {
    let input = validate_input(input)?;

    match fs::create_dir(&input.destination) {
        Ok(()) => {}
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
            return Err(CreateProjectError::DestinationExists(input.destination));
        }
        Err(source) => {
            return Err(io_error(
                "create the project directory",
                &input.destination,
                source,
            ));
        }
    }

    match initialize_workspace(&input, &project_id) {
        Ok(()) => Ok(CreatedProject {
            project_id,
            path: input.destination,
        }),
        Err(error) => Err(rollback_failed_creation(&input.destination, error)),
    }
}

fn validate_input(input: CreateProjectInput) -> Result<ValidatedCreateProject, CreateProjectError> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err(CreateProjectError::InvalidTitle);
    }

    let language = input.language.trim();
    if language.is_empty() {
        return Err(CreateProjectError::InvalidLanguage);
    }

    if !input.destination.is_absolute() {
        return Err(CreateProjectError::InvalidDestination(
            "the path must be absolute",
        ));
    }

    if input.destination.extension() != Some(OsStr::new(PROJECT_EXTENSION)) {
        return Err(CreateProjectError::InvalidDestination(
            "the directory must use the .bookmaker extension",
        ));
    }

    if input
        .destination
        .file_stem()
        .is_none_or(|stem| stem.is_empty())
    {
        return Err(CreateProjectError::InvalidDestination(
            "the directory must have a name",
        ));
    }

    let parent = input
        .destination
        .parent()
        .ok_or(CreateProjectError::InvalidDestination(
            "the path must have a parent directory",
        ))?;
    let canonical_parent = fs::canonicalize(parent)
        .map_err(|source| io_error("resolve the parent directory", parent, source))?;

    if !canonical_parent.is_dir() {
        return Err(CreateProjectError::InvalidDestination(
            "the parent path must be a directory",
        ));
    }

    let file_name = input
        .destination
        .file_name()
        .ok_or(CreateProjectError::InvalidDestination(
            "the directory must have a name",
        ))?;
    let destination = canonical_parent.join(file_name);

    match fs::symlink_metadata(&destination) {
        Ok(_) => return Err(CreateProjectError::DestinationExists(destination)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(io_error(
                "inspect the project destination",
                &destination,
                source,
            ));
        }
    }

    Ok(ValidatedCreateProject {
        destination,
        title: title.to_owned(),
        language: language.to_owned(),
    })
}

fn initialize_workspace(
    input: &ValidatedCreateProject,
    project_id: &str,
) -> Result<(), CreateProjectError> {
    for relative_path in REQUIRED_DIRECTORIES {
        let path = input.destination.join(relative_path);
        fs::create_dir_all(&path)
            .map_err(|source| io_error("create a project subdirectory", &path, source))?;
    }

    initialize_database(input, project_id)?;
    write_manifest(&input.destination, project_id)?;

    Ok(())
}

fn initialize_database(
    input: &ValidatedCreateProject,
    project_id: &str,
) -> Result<(), CreateProjectError> {
    let database_path = input.destination.join(DATABASE_FILE);
    let mut connection =
        Connection::open(&database_path).map_err(|source| CreateProjectError::Database {
            path: database_path.clone(),
            source,
        })?;

    run_migrations(&mut connection).map_err(CreateProjectError::Migration)?;

    let transaction = connection
        .transaction()
        .map_err(|source| CreateProjectError::Database {
            path: database_path.clone(),
            source,
        })?;
    transaction
        .execute(
            "INSERT INTO project_info (
                id, format_version, created_at, updated_at
             ) VALUES (
                ?1, ?2,
                strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             )",
            params![project_id, PROJECT_FORMAT_VERSION],
        )
        .map_err(|source| CreateProjectError::Database {
            path: database_path.clone(),
            source,
        })?;
    transaction
        .execute(
            "INSERT INTO project_metadata (
                project_id, title, language, rights_statement_mode
             ) VALUES (?1, ?2, ?3, 'generated')",
            params![project_id, input.title, input.language],
        )
        .map_err(|source| CreateProjectError::Database {
            path: database_path.clone(),
            source,
        })?;
    transaction
        .commit()
        .map_err(|source| CreateProjectError::Database {
            path: database_path,
            source,
        })?;

    Ok(())
}

fn write_manifest(destination: &Path, project_id: &str) -> Result<(), CreateProjectError> {
    let manifest = ProjectManifest::new(project_id.to_owned());
    let temporary_path = destination.join("manifest.json.tmp");
    let manifest_path = destination.join(MANIFEST_FILE);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary_path)
        .map_err(|source| io_error("create the temporary manifest", &temporary_path, source))?;

    serde_json::to_writer_pretty(&mut file, &manifest)
        .map_err(CreateProjectError::ManifestSerialization)?;
    file.write_all(b"\n")
        .map_err(|source| io_error("write the manifest", &temporary_path, source))?;
    file.sync_all()
        .map_err(|source| io_error("flush the manifest", &temporary_path, source))?;
    drop(file);

    fs::rename(&temporary_path, &manifest_path)
        .map_err(|source| io_error("commit the manifest", &manifest_path, source))?;

    Ok(())
}

fn rollback_failed_creation(
    destination: &Path,
    original: CreateProjectError,
) -> CreateProjectError {
    match fs::remove_dir_all(destination) {
        Ok(()) => original,
        Err(source) if source.kind() == io::ErrorKind::NotFound => original,
        Err(source) => CreateProjectError::RollbackFailed {
            destination: destination.to_owned(),
            original: Box::new(original),
            source,
        },
    }
}

fn io_error(operation: &'static str, path: &Path, source: io::Error) -> CreateProjectError {
    CreateProjectError::Io {
        operation,
        path: path.to_owned(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir()
                .join(format!("bookmaker-create-project-test-{}", Uuid::now_v7()));
            fs::create_dir(&path).expect("test parent directory must be created");
            Self { path }
        }

        fn project_path(&self) -> PathBuf {
            self.path.join("My Book.bookmaker")
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            if let Err(error) = fs::remove_dir_all(&self.path) {
                if error.kind() != io::ErrorKind::NotFound {
                    panic!("test directory cleanup failed: {error}");
                }
            }
        }
    }

    #[test]
    fn creates_a_complete_project_workspace() {
        let test_directory = TestDirectory::new();
        let destination = test_directory.project_path();

        let created = create_project(CreateProjectInput {
            destination: destination.clone(),
            title: "  Meu Livro — Café  ".to_owned(),
            language: " pt-BR ".to_owned(),
        })
        .expect("project creation must succeed");

        assert_eq!(
            created.path,
            fs::canonicalize(&destination).expect("project path must canonicalize")
        );
        assert_eq!(
            Uuid::parse_str(&created.project_id)
                .expect("project id must be a UUID")
                .get_version_num(),
            7
        );
        for relative_path in REQUIRED_DIRECTORIES {
            assert!(destination.join(relative_path).is_dir());
        }

        let manifest: Value = serde_json::from_slice(
            &fs::read(destination.join(MANIFEST_FILE)).expect("manifest must be readable"),
        )
        .expect("manifest must be valid JSON");
        assert_eq!(manifest["format"], PROJECT_FORMAT);
        assert_eq!(manifest["formatVersion"], PROJECT_FORMAT_VERSION);
        assert_eq!(manifest["projectId"], created.project_id);
        assert_eq!(manifest["createdBy"], PROJECT_CREATOR);
        assert_eq!(manifest["minimumAppVersion"], env!("CARGO_PKG_VERSION"));

        let connection =
            Connection::open(destination.join(DATABASE_FILE)).expect("database must open");
        let project: (String, i64, String, String) = connection
            .query_row(
                "SELECT project_info.id, project_info.format_version,
                        project_metadata.title, project_metadata.language
                 FROM project_info
                 JOIN project_metadata ON project_metadata.project_id = project_info.id",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("initial project data must be persisted");
        assert_eq!(project.0, created.project_id);
        assert_eq!(project.1, PROJECT_FORMAT_VERSION);
        assert_eq!(project.2, "Meu Livro — Café");
        assert_eq!(project.3, "pt-BR");

        let schema_version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("schema version must be recorded");
        assert_eq!(
            schema_version,
            crate::persistence::migrations::CURRENT_SCHEMA_VERSION
        );
    }

    #[test]
    fn preserves_an_existing_destination() {
        let test_directory = TestDirectory::new();
        let destination = test_directory.project_path();
        fs::create_dir(&destination).expect("existing directory must be created");
        let marker = destination.join("keep.txt");
        fs::write(&marker, "keep").expect("marker must be written");

        let error = create_project(CreateProjectInput {
            destination,
            title: "Book".to_owned(),
            language: "en".to_owned(),
        })
        .expect_err("existing destination must be rejected");

        assert!(matches!(error, CreateProjectError::DestinationExists(_)));
        assert_eq!(
            fs::read_to_string(marker).expect("marker must remain"),
            "keep"
        );
    }

    #[test]
    fn rejects_invalid_input_before_creating_a_directory() {
        let test_directory = TestDirectory::new();
        let destination = test_directory.project_path();

        let error = create_project(CreateProjectInput {
            destination: destination.clone(),
            title: "   ".to_owned(),
            language: "pt-BR".to_owned(),
        })
        .expect_err("blank title must be rejected");

        assert!(matches!(error, CreateProjectError::InvalidTitle));
        assert!(!destination.exists());
    }

    #[test]
    fn requires_the_bookmaker_extension() {
        let test_directory = TestDirectory::new();
        let destination = test_directory.path.join("My Book");

        let error = create_project(CreateProjectInput {
            destination: destination.clone(),
            title: "Book".to_owned(),
            language: "en".to_owned(),
        })
        .expect_err("destination without the project extension must be rejected");

        assert!(matches!(error, CreateProjectError::InvalidDestination(_)));
        assert!(!destination.exists());
    }

    #[test]
    fn removes_a_partial_workspace_after_initialization_fails() {
        let test_directory = TestDirectory::new();
        let destination = test_directory.project_path();
        fs::create_dir(&destination).expect("partial project must be created");
        fs::create_dir(destination.join(DATABASE_FILE))
            .expect("database path collision must be created");
        let input = ValidatedCreateProject {
            destination: destination.clone(),
            title: "Book".to_owned(),
            language: "en".to_owned(),
        };

        let initialization_error = initialize_workspace(&input, "project-id")
            .expect_err("database path collision must fail");
        let error = rollback_failed_creation(&destination, initialization_error);

        assert!(matches!(error, CreateProjectError::Database { .. }));
        assert!(!destination.exists());
    }
}
