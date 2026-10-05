use std::{
    error::Error,
    ffi::OsStr,
    fmt, fs,
    fs::File,
    io,
    path::{Path, PathBuf},
};

use rusqlite::{Connection, OpenFlags};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::persistence::{
    migrations::{run_migrations, MigrationError},
    project_manifest::{
        ProjectManifest, DATABASE_FILE, MANIFEST_FILE, PROJECT_CREATOR, PROJECT_EXTENSION,
        PROJECT_FORMAT, PROJECT_FORMAT_VERSION,
    },
};

#[derive(Debug, Deserialize)]
pub struct OpenProjectInput {
    pub path: PathBuf,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenedProject {
    pub project_id: String,
    pub path: PathBuf,
    pub title: String,
    pub language: String,
    pub format_version: i64,
    pub schema_version: i64,
}

#[derive(Debug)]
pub enum OpenProjectError {
    InvalidPath(&'static str),
    Io {
        operation: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    InvalidManifest(serde_json::Error),
    InvalidManifestValue(&'static str),
    UnsupportedFormatVersion {
        found: i64,
        supported: i64,
    },
    InvalidMinimumAppVersion(String),
    AppVersionTooOld {
        required: Version,
        current: Version,
    },
    Database {
        path: PathBuf,
        source: rusqlite::Error,
    },
    Migration(MigrationError),
    CorruptProject(&'static str),
}

impl OpenProjectError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidPath(_) => "project.invalid_path",
            Self::UnsupportedFormatVersion { .. } => "project.unsupported_format",
            Self::AppVersionTooOld { .. } => "project.app_too_old",
            Self::InvalidManifest(_)
            | Self::InvalidManifestValue(_)
            | Self::InvalidMinimumAppVersion(_) => "project.invalid_manifest",
            Self::CorruptProject(_) => "project.corrupt",
            Self::Io { .. } | Self::Database { .. } | Self::Migration(_) => "project.open_failed",
        }
    }

    pub fn user_message(&self) -> &'static str {
        match self {
            Self::InvalidPath(_) => "Escolha uma pasta de projeto válida.",
            Self::UnsupportedFormatVersion { .. } => {
                "Este projeto usa um formato ainda não suportado."
            }
            Self::AppVersionTooOld { .. } => "Atualize o BookMaker para abrir este projeto.",
            Self::InvalidManifest(_)
            | Self::InvalidManifestValue(_)
            | Self::InvalidMinimumAppVersion(_) => "O manifest do projeto é inválido.",
            Self::CorruptProject(_) => "Os dados do projeto estão inconsistentes.",
            Self::Io { .. } | Self::Database { .. } | Self::Migration(_) => {
                "Não foi possível abrir o projeto."
            }
        }
    }
}

impl fmt::Display for OpenProjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath(reason) => write!(formatter, "invalid project path: {reason}"),
            Self::Io {
                operation,
                path,
                source,
            } => write!(
                formatter,
                "failed to {operation} at {}: {source}",
                path.display()
            ),
            Self::InvalidManifest(source) => {
                write!(formatter, "invalid project manifest: {source}")
            }
            Self::InvalidManifestValue(reason) => {
                write!(formatter, "invalid project manifest: {reason}")
            }
            Self::UnsupportedFormatVersion { found, supported } => write!(
                formatter,
                "project format version {found} is not supported; current version is {supported}"
            ),
            Self::InvalidMinimumAppVersion(version) => {
                write!(formatter, "invalid minimum app version: {version}")
            }
            Self::AppVersionTooOld { required, current } => write!(
                formatter,
                "project requires BookMaker {required}, current version is {current}"
            ),
            Self::Database { path, source } => {
                write!(
                    formatter,
                    "database operation failed at {}: {source}",
                    path.display()
                )
            }
            Self::Migration(source) => write!(formatter, "project migration failed: {source}"),
            Self::CorruptProject(reason) => write!(formatter, "project is inconsistent: {reason}"),
        }
    }
}

impl Error for OpenProjectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidManifest(source) => Some(source),
            Self::Database { source, .. } => Some(source),
            Self::Migration(source) => Some(source),
            _ => None,
        }
    }
}

pub fn open_project(input: OpenProjectInput) -> Result<OpenedProject, OpenProjectError> {
    let project_path = validate_project_path(&input.path)?;
    let manifest = read_manifest(&project_path)?;
    validate_manifest(&manifest)?;

    let database_path = project_path.join(DATABASE_FILE);
    validate_database_path(&database_path)?;
    let mut connection =
        Connection::open_with_flags(&database_path, OpenFlags::SQLITE_OPEN_READ_WRITE).map_err(
            |source| OpenProjectError::Database {
                path: database_path.clone(),
                source,
            },
        )?;
    let migration_report = run_migrations(&mut connection).map_err(OpenProjectError::Migration)?;

    let project_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM project_info", [], |row| row.get(0))
        .map_err(|source| database_error(&database_path, source))?;
    if project_count != 1 {
        return Err(OpenProjectError::CorruptProject(
            "the database must contain exactly one project",
        ));
    }

    let project: (String, i64, String, String) = connection
        .query_row(
            "SELECT project_info.id, project_info.format_version,
                    project_metadata.title, project_metadata.language
             FROM project_info
             JOIN project_metadata ON project_metadata.project_id = project_info.id",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|source| database_error(&database_path, source))?;

    if project.0 != manifest.project_id {
        return Err(OpenProjectError::CorruptProject(
            "manifest and database project IDs differ",
        ));
    }
    if project.1 != manifest.format_version {
        return Err(OpenProjectError::CorruptProject(
            "manifest and database format versions differ",
        ));
    }

    Ok(OpenedProject {
        project_id: project.0,
        path: project_path,
        title: project.2,
        language: project.3,
        format_version: project.1,
        schema_version: migration_report.current_version,
    })
}

fn validate_project_path(path: &Path) -> Result<PathBuf, OpenProjectError> {
    if !path.is_absolute() {
        return Err(OpenProjectError::InvalidPath("the path must be absolute"));
    }
    if path.extension() != Some(OsStr::new(PROJECT_EXTENSION)) {
        return Err(OpenProjectError::InvalidPath(
            "the directory must use the .bookmaker extension",
        ));
    }

    let metadata = fs::symlink_metadata(path)
        .map_err(|source| io_error("inspect the project directory", path, source))?;
    if metadata.file_type().is_symlink() {
        return Err(OpenProjectError::InvalidPath(
            "symbolic links are not accepted as project roots",
        ));
    }
    if !metadata.is_dir() {
        return Err(OpenProjectError::InvalidPath(
            "the project path must be a directory",
        ));
    }

    fs::canonicalize(path).map_err(|source| io_error("resolve the project directory", path, source))
}

fn read_manifest(project_path: &Path) -> Result<ProjectManifest, OpenProjectError> {
    let manifest_path = project_path.join(MANIFEST_FILE);
    let file = File::open(&manifest_path)
        .map_err(|source| io_error("open the project manifest", &manifest_path, source))?;
    serde_json::from_reader(file).map_err(OpenProjectError::InvalidManifest)
}

fn validate_manifest(manifest: &ProjectManifest) -> Result<(), OpenProjectError> {
    if manifest.format != PROJECT_FORMAT {
        return Err(OpenProjectError::InvalidManifestValue(
            "unknown project format",
        ));
    }
    if manifest.format_version != PROJECT_FORMAT_VERSION {
        return Err(OpenProjectError::UnsupportedFormatVersion {
            found: manifest.format_version,
            supported: PROJECT_FORMAT_VERSION,
        });
    }
    if manifest.project_id.trim().is_empty() {
        return Err(OpenProjectError::InvalidManifestValue(
            "project ID cannot be blank",
        ));
    }
    if manifest.created_by != PROJECT_CREATOR {
        return Err(OpenProjectError::InvalidManifestValue(
            "unknown project creator",
        ));
    }

    let required = Version::parse(&manifest.minimum_app_version).map_err(|_| {
        OpenProjectError::InvalidMinimumAppVersion(manifest.minimum_app_version.clone())
    })?;
    let current = Version::parse(env!("CARGO_PKG_VERSION"))
        .expect("Cargo package version must be valid semantic versioning");
    if current < required {
        return Err(OpenProjectError::AppVersionTooOld { required, current });
    }

    Ok(())
}

fn validate_database_path(path: &Path) -> Result<(), OpenProjectError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|source| io_error("inspect the project database", path, source))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(OpenProjectError::InvalidPath(
            "project.db must be a regular file",
        ));
    }
    Ok(())
}

fn io_error(operation: &'static str, path: &Path, source: io::Error) -> OpenProjectError {
    OpenProjectError::Io {
        operation,
        path: path.to_owned(),
        source,
    }
}

fn database_error(path: &Path, source: rusqlite::Error) -> OpenProjectError {
    OpenProjectError::Database {
        path: path.to_owned(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::create_project::{create_project, CreateProjectInput};
    use serde_json::{json, Value};
    use uuid::Uuid;

    struct TestProject {
        parent: PathBuf,
        path: PathBuf,
    }

    impl TestProject {
        fn create() -> Self {
            let parent = std::env::temp_dir()
                .join(format!("bookmaker-open-project-test-{}", Uuid::now_v7()));
            fs::create_dir(&parent).expect("test parent must be created");
            let path = parent.join("Project.bookmaker");
            create_project(CreateProjectInput {
                destination: path.clone(),
                title: "Projeto — Teste".to_owned(),
                language: "pt-BR".to_owned(),
            })
            .expect("test project must be created");
            Self { parent, path }
        }

        fn update_manifest(&self, update: impl FnOnce(&mut Value)) {
            let path = self.path.join(MANIFEST_FILE);
            let mut manifest: Value =
                serde_json::from_slice(&fs::read(&path).expect("manifest must be readable"))
                    .expect("manifest must be JSON");
            update(&mut manifest);
            fs::write(
                path,
                serde_json::to_vec_pretty(&manifest).expect("manifest must serialize"),
            )
            .expect("manifest must be updated");
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
    fn opens_a_valid_project() {
        let project = TestProject::create();

        let opened = open_project(OpenProjectInput {
            path: project.path.clone(),
        })
        .expect("valid project must open");

        assert_eq!(opened.path, fs::canonicalize(&project.path).unwrap());
        assert_eq!(opened.title, "Projeto — Teste");
        assert_eq!(opened.language, "pt-BR");
        assert_eq!(opened.format_version, PROJECT_FORMAT_VERSION);
        assert_eq!(
            opened.schema_version,
            crate::persistence::migrations::CURRENT_SCHEMA_VERSION
        );
    }

    #[test]
    fn rejects_unknown_manifest_fields() {
        let project = TestProject::create();
        project.update_manifest(|manifest| manifest["unexpected"] = json!(true));

        let error = open_project(OpenProjectInput {
            path: project.path.clone(),
        })
        .expect_err("unknown fields must be rejected");

        assert!(matches!(error, OpenProjectError::InvalidManifest(_)));
    }

    #[test]
    fn rejects_a_manifest_and_database_id_mismatch() {
        let project = TestProject::create();
        project
            .update_manifest(|manifest| manifest["projectId"] = json!(Uuid::now_v7().to_string()));

        let error = open_project(OpenProjectInput {
            path: project.path.clone(),
        })
        .expect_err("mismatched project IDs must be rejected");

        assert!(matches!(error, OpenProjectError::CorruptProject(_)));
    }

    #[test]
    fn rejects_a_project_that_requires_a_newer_application() {
        let project = TestProject::create();
        project.update_manifest(|manifest| manifest["minimumAppVersion"] = json!("999.0.0"));

        let error = open_project(OpenProjectInput {
            path: project.path.clone(),
        })
        .expect_err("newer application requirement must be rejected");

        assert!(matches!(error, OpenProjectError::AppVersionTooOld { .. }));
    }
}
