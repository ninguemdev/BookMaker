use std::{
    error::Error,
    fmt, fs,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, SystemTimeError, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

const STORE_VERSION: u32 = 1;
const STORE_FILE: &str = "recent-projects.json";
const TEMPORARY_FILE: &str = "recent-projects.json.tmp";
const BACKUP_FILE: &str = "recent-projects.json.backup";

#[derive(Debug, Clone)]
pub struct RecentProjectUpdate {
    pub project_id: String,
    pub path: PathBuf,
    pub title: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentProject {
    pub project_id: String,
    pub path: PathBuf,
    pub title: String,
    pub last_opened_at_ms: u64,
    pub available: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RecentProjectStore {
    version: u32,
    projects: Vec<StoredRecentProject>,
}

impl Default for RecentProjectStore {
    fn default() -> Self {
        Self {
            version: STORE_VERSION,
            projects: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredRecentProject {
    project_id: String,
    path: PathBuf,
    title: String,
    last_opened_at_ms: u64,
}

#[derive(Debug)]
pub enum RecentProjectsError {
    Io {
        operation: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    InvalidStore(serde_json::Error),
    UnsupportedStoreVersion(u32),
    Clock(SystemTimeError),
    ReplaceFailed {
        source: io::Error,
        restore_error: Option<io::Error>,
    },
}

impl RecentProjectsError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidStore(_) | Self::UnsupportedStoreVersion(_) => {
                "recent_projects.invalid_store"
            }
            Self::Io { .. } | Self::Clock(_) | Self::ReplaceFailed { .. } => {
                "recent_projects.unavailable"
            }
        }
    }

    pub fn user_message(&self) -> &'static str {
        "Não foi possível atualizar a lista de projetos recentes."
    }
}

impl fmt::Display for RecentProjectsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                operation,
                path,
                source,
            } => write!(
                formatter,
                "failed to {operation} at {}: {source}",
                path.display()
            ),
            Self::InvalidStore(source) => {
                write!(formatter, "invalid recent projects store: {source}")
            }
            Self::UnsupportedStoreVersion(version) => {
                write!(
                    formatter,
                    "unsupported recent projects store version: {version}"
                )
            }
            Self::Clock(source) => write!(formatter, "system clock is invalid: {source}"),
            Self::ReplaceFailed {
                source,
                restore_error,
            } => {
                write!(
                    formatter,
                    "failed to replace the recent projects store: {source}"
                )?;
                if let Some(restore_error) = restore_error {
                    write!(
                        formatter,
                        "; restoring the previous store also failed: {restore_error}"
                    )?;
                }
                Ok(())
            }
        }
    }
}

impl Error for RecentProjectsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidStore(source) => Some(source),
            Self::Clock(source) => Some(source),
            Self::ReplaceFailed { source, .. } => Some(source),
            Self::UnsupportedStoreVersion(_) => None,
        }
    }
}

pub fn list_recent_projects(
    application_data_directory: &Path,
) -> Result<Vec<RecentProject>, RecentProjectsError> {
    let store = load_store(application_data_directory)?;
    Ok(store
        .projects
        .into_iter()
        .map(|project| RecentProject {
            available: project.path.is_dir(),
            project_id: project.project_id,
            path: project.path,
            title: project.title,
            last_opened_at_ms: project.last_opened_at_ms,
        })
        .collect())
}

pub fn record_recent_project(
    application_data_directory: &Path,
    project: RecentProjectUpdate,
) -> Result<(), RecentProjectsError> {
    let mut store = load_store(application_data_directory)?;
    store.projects.retain(|existing| {
        existing.project_id != project.project_id && existing.path != project.path
    });
    store.projects.insert(
        0,
        StoredRecentProject {
            project_id: project.project_id,
            path: project.path,
            title: project.title,
            last_opened_at_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(RecentProjectsError::Clock)?
                .as_millis()
                .try_into()
                .unwrap_or(u64::MAX),
        },
    );
    write_store(application_data_directory, &store)
}

fn load_store(
    application_data_directory: &Path,
) -> Result<RecentProjectStore, RecentProjectsError> {
    let store_path = application_data_directory.join(STORE_FILE);
    if !store_path.exists() {
        restore_backup_if_needed(application_data_directory)?;
    }
    if !store_path.exists() {
        return Ok(RecentProjectStore::default());
    }

    let file = File::open(&store_path)
        .map_err(|source| io_error("open the recent projects store", &store_path, source))?;
    let store: RecentProjectStore =
        serde_json::from_reader(file).map_err(RecentProjectsError::InvalidStore)?;
    if store.version != STORE_VERSION {
        return Err(RecentProjectsError::UnsupportedStoreVersion(store.version));
    }
    Ok(store)
}

fn write_store(
    application_data_directory: &Path,
    store: &RecentProjectStore,
) -> Result<(), RecentProjectsError> {
    fs::create_dir_all(application_data_directory).map_err(|source| {
        io_error(
            "create the application data directory",
            application_data_directory,
            source,
        )
    })?;

    let temporary_path = application_data_directory.join(TEMPORARY_FILE);
    let mut temporary_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary_path)
        .map_err(|source| {
            io_error(
                "create the temporary recent projects store",
                &temporary_path,
                source,
            )
        })?;
    serde_json::to_writer_pretty(&mut temporary_file, store)
        .map_err(RecentProjectsError::InvalidStore)?;
    temporary_file
        .write_all(b"\n")
        .map_err(|source| io_error("write the recent projects store", &temporary_path, source))?;
    temporary_file
        .sync_all()
        .map_err(|source| io_error("flush the recent projects store", &temporary_path, source))?;
    drop(temporary_file);

    replace_store(application_data_directory)
}

fn replace_store(application_data_directory: &Path) -> Result<(), RecentProjectsError> {
    let store_path = application_data_directory.join(STORE_FILE);
    let temporary_path = application_data_directory.join(TEMPORARY_FILE);
    let backup_path = application_data_directory.join(BACKUP_FILE);

    if !store_path.exists() {
        return fs::rename(&temporary_path, &store_path)
            .map_err(|source| io_error("commit the recent projects store", &store_path, source));
    }

    if backup_path.exists() {
        fs::remove_file(&backup_path).map_err(|source| {
            io_error(
                "remove a stale recent projects backup",
                &backup_path,
                source,
            )
        })?;
    }
    fs::rename(&store_path, &backup_path)
        .map_err(|source| io_error("backup the recent projects store", &store_path, source))?;

    if let Err(source) = fs::rename(&temporary_path, &store_path) {
        let restore_error = fs::rename(&backup_path, &store_path).err();
        return Err(RecentProjectsError::ReplaceFailed {
            source,
            restore_error,
        });
    }

    fs::remove_file(&backup_path)
        .map_err(|source| io_error("remove the recent projects backup", &backup_path, source))?;
    Ok(())
}

fn restore_backup_if_needed(application_data_directory: &Path) -> Result<(), RecentProjectsError> {
    let store_path = application_data_directory.join(STORE_FILE);
    let backup_path = application_data_directory.join(BACKUP_FILE);
    if backup_path.exists() {
        fs::rename(&backup_path, &store_path).map_err(|source| {
            io_error("restore the recent projects store", &backup_path, source)
        })?;
    }
    Ok(())
}

fn io_error(operation: &'static str, path: &Path, source: io::Error) -> RecentProjectsError {
    RecentProjectsError::Io {
        operation,
        path: path.to_owned(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir()
                .join(format!("bookmaker-recent-projects-test-{}", Uuid::now_v7()));
            fs::create_dir(&path).expect("test directory must be created");
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            if let Err(error) = fs::remove_dir_all(&self.0) {
                if error.kind() != io::ErrorKind::NotFound {
                    panic!("test cleanup failed: {error}");
                }
            }
        }
    }

    #[test]
    fn returns_an_empty_list_before_the_store_exists() {
        let directory = TestDirectory::new();

        assert!(list_recent_projects(&directory.0).unwrap().is_empty());
    }

    #[test]
    fn records_newest_first_and_deduplicates_projects() {
        let directory = TestDirectory::new();
        let first_path = directory.0.join("First.bookmaker");
        let second_path = directory.0.join("Second.bookmaker");
        fs::create_dir(&first_path).unwrap();
        fs::create_dir(&second_path).unwrap();

        record_recent_project(
            &directory.0,
            RecentProjectUpdate {
                project_id: "project-1".to_owned(),
                path: first_path.clone(),
                title: "First".to_owned(),
            },
        )
        .unwrap();
        record_recent_project(
            &directory.0,
            RecentProjectUpdate {
                project_id: "project-2".to_owned(),
                path: second_path,
                title: "Second".to_owned(),
            },
        )
        .unwrap();
        record_recent_project(
            &directory.0,
            RecentProjectUpdate {
                project_id: "project-1".to_owned(),
                path: first_path,
                title: "First renamed".to_owned(),
            },
        )
        .unwrap();

        let recent = list_recent_projects(&directory.0).unwrap();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].project_id, "project-1");
        assert_eq!(recent[0].title, "First renamed");
        assert_eq!(recent[1].project_id, "project-2");
        assert!(recent.iter().all(|project| project.available));
    }

    #[test]
    fn keeps_missing_projects_and_marks_them_unavailable() {
        let directory = TestDirectory::new();
        record_recent_project(
            &directory.0,
            RecentProjectUpdate {
                project_id: "missing".to_owned(),
                path: directory.0.join("Missing.bookmaker"),
                title: "Missing".to_owned(),
            },
        )
        .unwrap();

        let recent = list_recent_projects(&directory.0).unwrap();
        assert_eq!(recent.len(), 1);
        assert!(!recent[0].available);
    }

    #[test]
    fn restores_the_previous_store_after_an_interrupted_replace() {
        let directory = TestDirectory::new();
        let store = RecentProjectStore {
            version: STORE_VERSION,
            projects: vec![StoredRecentProject {
                project_id: "recovered".to_owned(),
                path: directory.0.join("Recovered.bookmaker"),
                title: "Recovered".to_owned(),
                last_opened_at_ms: 1,
            }],
        };
        fs::write(
            directory.0.join(BACKUP_FILE),
            serde_json::to_vec(&store).unwrap(),
        )
        .unwrap();

        let recent = list_recent_projects(&directory.0).unwrap();

        assert_eq!(recent[0].project_id, "recovered");
        assert!(directory.0.join(STORE_FILE).is_file());
        assert!(!directory.0.join(BACKUP_FILE).exists());
    }
}
