use std::{
    error::Error,
    ffi::OsStr,
    fmt, fs,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, SystemTimeError, UNIX_EPOCH},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::persistence::project_manifest::PROJECT_EXTENSION;

const JOURNAL_VERSION: u32 = 1;
const RECOVERY_DIRECTORY: &str = "recovery";
const DOCUMENTS_DIRECTORY: &str = "documents";
const SESSION_FILE: &str = "session.json";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecoverySession {
    pub version: u32,
    pub session_id: String,
    pub started_at_ms: u64,
    pub process_id: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndRecoverySessionInput {
    pub project_path: PathBuf,
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteRecoveryCheckpointInput {
    pub project_path: PathBuf,
    pub session_id: String,
    pub document_id: String,
    pub schema_version: u32,
    pub persisted_updated_at: Option<String>,
    pub content: Value,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecoveryCheckpoint {
    pub version: u32,
    pub document_id: String,
    pub schema_version: u32,
    pub captured_at_ms: u64,
    pub persisted_updated_at: Option<String>,
    pub content: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryDocumentInput {
    pub project_path: PathBuf,
    pub session_id: String,
    pub document_id: String,
}

#[derive(Debug)]
pub enum RecoveryJournalError {
    InvalidProjectPath(&'static str),
    InvalidDocumentId,
    InvalidSchemaVersion,
    Io {
        operation: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    InvalidJournal {
        path: PathBuf,
        source: serde_json::Error,
    },
    ExistingSession(RecoverySession),
    SessionNotStarted,
    SessionMismatch,
    UnsupportedJournalVersion(u32),
    Clock(SystemTimeError),
    ReplaceFailed {
        path: PathBuf,
        source: io::Error,
        restore_error: Option<io::Error>,
    },
}

impl RecoveryJournalError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidProjectPath(_) | Self::InvalidDocumentId | Self::InvalidSchemaVersion => {
                "recovery.invalid_input"
            }
            Self::ExistingSession(_) => "recovery.session_exists",
            Self::SessionNotStarted | Self::SessionMismatch => "recovery.session_mismatch",
            Self::InvalidJournal { .. } | Self::UnsupportedJournalVersion(_) => {
                "recovery.invalid_journal"
            }
            Self::Io { .. } | Self::Clock(_) | Self::ReplaceFailed { .. } => "recovery.unavailable",
        }
    }

    pub fn user_message(&self) -> &'static str {
        match self {
            Self::ExistingSession(_) => {
                "O projeto já possui uma sessão aberta ou não encerrada corretamente."
            }
            Self::SessionNotStarted | Self::SessionMismatch => {
                "A sessão de recuperação do projeto não é válida."
            }
            Self::InvalidJournal { .. } | Self::UnsupportedJournalVersion(_) => {
                "Os dados de recuperação do projeto estão inválidos."
            }
            Self::InvalidProjectPath(_) | Self::InvalidDocumentId | Self::InvalidSchemaVersion => {
                "Os dados enviados para recuperação são inválidos."
            }
            Self::Io { .. } | Self::Clock(_) | Self::ReplaceFailed { .. } => {
                "Não foi possível atualizar os dados de recuperação."
            }
        }
    }
}

impl fmt::Display for RecoveryJournalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProjectPath(reason) => write!(formatter, "invalid project path: {reason}"),
            Self::InvalidDocumentId => formatter.write_str("invalid recovery document ID"),
            Self::InvalidSchemaVersion => {
                formatter.write_str("recovery schema version must be greater than zero")
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
            Self::InvalidJournal { path, source } => {
                write!(
                    formatter,
                    "invalid recovery data at {}: {source}",
                    path.display()
                )
            }
            Self::ExistingSession(session) => write!(
                formatter,
                "recovery session {} already exists for process {}",
                session.session_id, session.process_id
            ),
            Self::SessionNotStarted => formatter.write_str("recovery session has not been started"),
            Self::SessionMismatch => formatter.write_str("recovery session ID does not match"),
            Self::UnsupportedJournalVersion(version) => {
                write!(formatter, "unsupported recovery journal version: {version}")
            }
            Self::Clock(source) => write!(formatter, "system clock is invalid: {source}"),
            Self::ReplaceFailed {
                path,
                source,
                restore_error,
            } => {
                write!(formatter, "failed to replace {}: {source}", path.display())?;
                if let Some(restore_error) = restore_error {
                    write!(
                        formatter,
                        "; restoring the previous file failed: {restore_error}"
                    )?;
                }
                Ok(())
            }
        }
    }
}

impl Error for RecoveryJournalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidJournal { source, .. } => Some(source),
            Self::Clock(source) => Some(source),
            Self::ReplaceFailed { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub fn begin_recovery_session(
    project_path: &Path,
) -> Result<RecoverySession, RecoveryJournalError> {
    let recovery_path = validated_recovery_path(project_path)?;
    let session_path = recovery_path.join(SESSION_FILE);
    let session = RecoverySession {
        version: JOURNAL_VERSION,
        session_id: Uuid::now_v7().to_string(),
        started_at_ms: now_ms()?,
        process_id: std::process::id(),
    };

    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&session_path)
    {
        Ok(mut file) => {
            if let Err(error) = write_json(&mut file, &session, &session_path) {
                drop(file);
                let _ = fs::remove_file(&session_path);
                return Err(error);
            }
            Ok(session)
        }
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
            validate_regular_file_if_present(
                &session_path,
                "the recovery session must be a regular file",
            )?;
            let session: RecoverySession = read_json(&session_path)?;
            validate_version(session.version)?;
            Err(RecoveryJournalError::ExistingSession(session))
        }
        Err(source) => Err(io_error(
            "create the recovery session",
            &session_path,
            source,
        )),
    }
}

pub fn end_recovery_session(input: EndRecoverySessionInput) -> Result<(), RecoveryJournalError> {
    let recovery_path = validated_recovery_path(&input.project_path)?;
    validate_session(&recovery_path, &input.session_id)?;
    let session_path = recovery_path.join(SESSION_FILE);
    fs::remove_file(&session_path)
        .map_err(|source| io_error("remove the recovery session", &session_path, source))
}

pub fn write_recovery_checkpoint(
    input: WriteRecoveryCheckpointInput,
) -> Result<RecoveryCheckpoint, RecoveryJournalError> {
    validate_document_id(&input.document_id)?;
    if input.schema_version == 0 {
        return Err(RecoveryJournalError::InvalidSchemaVersion);
    }

    let recovery_path = validated_recovery_path(&input.project_path)?;
    validate_session(&recovery_path, &input.session_id)?;
    let documents_path = recovery_path.join(DOCUMENTS_DIRECTORY);
    prepare_documents_directory(&documents_path)?;

    let checkpoint = RecoveryCheckpoint {
        version: JOURNAL_VERSION,
        document_id: input.document_id.clone(),
        schema_version: input.schema_version,
        captured_at_ms: now_ms()?,
        persisted_updated_at: input.persisted_updated_at,
        content: input.content,
    };
    let checkpoint_path = checkpoint_path(&documents_path, &input.document_id);
    validate_regular_file_if_present(
        &checkpoint_path,
        "a recovery checkpoint must be a regular file",
    )?;
    write_json_atomically(&checkpoint_path, &checkpoint)?;
    Ok(checkpoint)
}

pub fn list_recovery_checkpoints(
    project_path: &Path,
) -> Result<Vec<RecoveryCheckpoint>, RecoveryJournalError> {
    let recovery_path = validated_recovery_path(project_path)?;
    let documents_path = recovery_path.join(DOCUMENTS_DIRECTORY);
    if !documents_path.exists() {
        return Ok(Vec::new());
    }
    validate_documents_directory(&documents_path)?;

    let mut checkpoints = Vec::new();
    let entries = fs::read_dir(&documents_path)
        .map_err(|source| io_error("list recovery checkpoints", &documents_path, source))?;
    for entry in entries {
        let entry = entry.map_err(|source| {
            io_error("read a recovery checkpoint entry", &documents_path, source)
        })?;
        let path = entry.path();
        if path.extension() != Some(OsStr::new("json"))
            || !entry
                .file_type()
                .map_err(|source| io_error("inspect a recovery checkpoint", &path, source))?
                .is_file()
        {
            continue;
        }

        let checkpoint: RecoveryCheckpoint = read_json(&path)?;
        validate_version(checkpoint.version)?;
        validate_document_id(&checkpoint.document_id)?;
        if checkpoint.schema_version == 0 {
            return Err(RecoveryJournalError::InvalidSchemaVersion);
        }
        checkpoints.push(checkpoint);
    }
    checkpoints.sort_by_key(|checkpoint| checkpoint.captured_at_ms);
    Ok(checkpoints)
}

pub fn clear_recovery_checkpoint(input: RecoveryDocumentInput) -> Result<(), RecoveryJournalError> {
    validate_document_id(&input.document_id)?;
    let recovery_path = validated_recovery_path(&input.project_path)?;
    validate_session(&recovery_path, &input.session_id)?;
    let documents_path = recovery_path.join(DOCUMENTS_DIRECTORY);
    if !documents_path.exists() {
        return Ok(());
    }
    validate_documents_directory(&documents_path)?;
    let path = checkpoint_path(&documents_path, &input.document_id);
    validate_regular_file_if_present(&path, "a recovery checkpoint must be a regular file")?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(io_error("remove the recovery checkpoint", &path, source)),
    }
}

fn validated_recovery_path(project_path: &Path) -> Result<PathBuf, RecoveryJournalError> {
    if !project_path.is_absolute() {
        return Err(RecoveryJournalError::InvalidProjectPath(
            "the path must be absolute",
        ));
    }
    if project_path.extension() != Some(OsStr::new(PROJECT_EXTENSION)) {
        return Err(RecoveryJournalError::InvalidProjectPath(
            "the directory must use the .bookmaker extension",
        ));
    }
    let project_metadata = fs::symlink_metadata(project_path)
        .map_err(|source| io_error("inspect the project directory", project_path, source))?;
    if project_metadata.file_type().is_symlink() || !project_metadata.is_dir() {
        return Err(RecoveryJournalError::InvalidProjectPath(
            "the project root must be a real directory",
        ));
    }

    let canonical_project = fs::canonicalize(project_path)
        .map_err(|source| io_error("resolve the project directory", project_path, source))?;
    let recovery_path = canonical_project.join(RECOVERY_DIRECTORY);
    let recovery_metadata = fs::symlink_metadata(&recovery_path)
        .map_err(|source| io_error("inspect the recovery directory", &recovery_path, source))?;
    if recovery_metadata.file_type().is_symlink() || !recovery_metadata.is_dir() {
        return Err(RecoveryJournalError::InvalidProjectPath(
            "the recovery path must be a real directory",
        ));
    }
    Ok(recovery_path)
}

fn validate_session(
    recovery_path: &Path,
    expected_session_id: &str,
) -> Result<(), RecoveryJournalError> {
    let session_path = recovery_path.join(SESSION_FILE);
    validate_regular_file_if_present(&session_path, "the recovery session must be a regular file")?;
    let session: RecoverySession = match read_json(&session_path) {
        Ok(session) => session,
        Err(RecoveryJournalError::Io { source, .. })
            if source.kind() == io::ErrorKind::NotFound =>
        {
            return Err(RecoveryJournalError::SessionNotStarted);
        }
        Err(error) => return Err(error),
    };
    validate_version(session.version)?;
    if session.session_id != expected_session_id {
        return Err(RecoveryJournalError::SessionMismatch);
    }
    Ok(())
}

fn validate_document_id(document_id: &str) -> Result<(), RecoveryJournalError> {
    if document_id.is_empty()
        || document_id.len() > 128
        || !document_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(RecoveryJournalError::InvalidDocumentId);
    }
    Ok(())
}

fn validate_version(version: u32) -> Result<(), RecoveryJournalError> {
    if version != JOURNAL_VERSION {
        return Err(RecoveryJournalError::UnsupportedJournalVersion(version));
    }
    Ok(())
}

fn checkpoint_path(documents_path: &Path, document_id: &str) -> PathBuf {
    documents_path.join(format!("{document_id}.json"))
}

fn prepare_documents_directory(path: &Path) -> Result<(), RecoveryJournalError> {
    fs::create_dir_all(path)
        .map_err(|source| io_error("create the recovery documents directory", path, source))?;
    validate_documents_directory(path)
}

fn validate_documents_directory(path: &Path) -> Result<(), RecoveryJournalError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|source| io_error("inspect the recovery documents directory", path, source))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(RecoveryJournalError::InvalidProjectPath(
            "the recovery documents path must be a real directory",
        ));
    }
    Ok(())
}

fn validate_regular_file_if_present(
    path: &Path,
    reason: &'static str,
) -> Result<(), RecoveryJournalError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(RecoveryJournalError::InvalidProjectPath(reason))
        }
        Ok(_) => Ok(()),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(io_error("inspect recovery data", path, source)),
    }
}

fn write_json_atomically<T: Serialize>(path: &Path, value: &T) -> Result<(), RecoveryJournalError> {
    let temporary_path = path.with_extension("json.tmp");
    let backup_path = path.with_extension("json.backup");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary_path)
        .map_err(|source| io_error("create a temporary recovery file", &temporary_path, source))?;
    write_json(&mut file, value, &temporary_path)?;

    if !path.exists() {
        return fs::rename(&temporary_path, path)
            .map_err(|source| io_error("commit a recovery file", path, source));
    }
    if backup_path.exists() {
        fs::remove_file(&backup_path)
            .map_err(|source| io_error("remove a stale recovery backup", &backup_path, source))?;
    }
    fs::rename(path, &backup_path)
        .map_err(|source| io_error("backup a recovery file", path, source))?;
    if let Err(source) = fs::rename(&temporary_path, path) {
        let restore_error = fs::rename(&backup_path, path).err();
        return Err(RecoveryJournalError::ReplaceFailed {
            path: path.to_owned(),
            source,
            restore_error,
        });
    }
    fs::remove_file(&backup_path)
        .map_err(|source| io_error("remove a recovery backup", &backup_path, source))?;
    Ok(())
}

fn write_json<T: Serialize>(
    file: &mut File,
    value: &T,
    path: &Path,
) -> Result<(), RecoveryJournalError> {
    serde_json::to_writer_pretty(&mut *file, value).map_err(|source| {
        RecoveryJournalError::InvalidJournal {
            path: path.to_owned(),
            source,
        }
    })?;
    file.write_all(b"\n")
        .map_err(|source| io_error("write recovery data", path, source))?;
    file.sync_all()
        .map_err(|source| io_error("flush recovery data", path, source))
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, RecoveryJournalError> {
    let file = File::open(path).map_err(|source| io_error("open recovery data", path, source))?;
    serde_json::from_reader(file).map_err(|source| RecoveryJournalError::InvalidJournal {
        path: path.to_owned(),
        source,
    })
}

fn now_ms() -> Result<u64, RecoveryJournalError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(RecoveryJournalError::Clock)?
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX))
}

fn io_error(operation: &'static str, path: &Path, source: io::Error) -> RecoveryJournalError {
    RecoveryJournalError::Io {
        operation,
        path: path.to_owned(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProject {
        parent: PathBuf,
        project: PathBuf,
    }

    impl TestProject {
        fn new() -> Self {
            let parent =
                std::env::temp_dir().join(format!("bookmaker-recovery-test-{}", Uuid::now_v7()));
            let project = parent.join("Recovery.bookmaker");
            fs::create_dir_all(project.join(RECOVERY_DIRECTORY))
                .expect("recovery directory must be created");
            Self { parent, project }
        }
    }

    impl Drop for TestProject {
        fn drop(&mut self) {
            if let Err(error) = fs::remove_dir_all(&self.parent) {
                if error.kind() != io::ErrorKind::NotFound {
                    panic!("test cleanup failed: {error}");
                }
            }
        }
    }

    #[test]
    fn detects_an_existing_session_and_requires_ownership_to_end_it() {
        let project = TestProject::new();
        let session = begin_recovery_session(&project.project).unwrap();

        let existing =
            begin_recovery_session(&project.project).expect_err("second session must be rejected");
        assert!(matches!(existing, RecoveryJournalError::ExistingSession(_)));

        let mismatch = end_recovery_session(EndRecoverySessionInput {
            project_path: project.project.clone(),
            session_id: "other-session".to_owned(),
        })
        .expect_err("another session cannot remove the marker");
        assert!(matches!(mismatch, RecoveryJournalError::SessionMismatch));

        end_recovery_session(EndRecoverySessionInput {
            project_path: project.project.clone(),
            session_id: session.session_id,
        })
        .unwrap();
        assert!(!project
            .project
            .join(RECOVERY_DIRECTORY)
            .join(SESSION_FILE)
            .exists());
    }

    #[test]
    fn writes_replaces_lists_and_clears_document_checkpoints() {
        let project = TestProject::new();
        let session = begin_recovery_session(&project.project).unwrap();
        let document_id = "0199b8ec-2e71-7000-8000-000000000002";

        write_recovery_checkpoint(WriteRecoveryCheckpointInput {
            project_path: project.project.clone(),
            session_id: session.session_id.clone(),
            document_id: document_id.to_owned(),
            schema_version: 1,
            persisted_updated_at: Some("2026-10-05T18:00:00Z".to_owned()),
            content: serde_json::json!({"type": "doc", "text": "olá"}),
        })
        .unwrap();
        write_recovery_checkpoint(WriteRecoveryCheckpointInput {
            project_path: project.project.clone(),
            session_id: session.session_id.clone(),
            document_id: document_id.to_owned(),
            schema_version: 1,
            persisted_updated_at: Some("2026-10-05T18:00:00Z".to_owned()),
            content: serde_json::json!({"type": "doc", "text": "mais recente"}),
        })
        .unwrap();

        let checkpoints = list_recovery_checkpoints(&project.project).unwrap();
        assert_eq!(checkpoints.len(), 1);
        assert_eq!(checkpoints[0].content["text"], "mais recente");

        clear_recovery_checkpoint(RecoveryDocumentInput {
            project_path: project.project.clone(),
            session_id: session.session_id,
            document_id: document_id.to_owned(),
        })
        .unwrap();
        assert!(list_recovery_checkpoints(&project.project)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn rejects_document_ids_that_could_escape_the_recovery_directory() {
        let project = TestProject::new();
        let session = begin_recovery_session(&project.project).unwrap();

        let error = write_recovery_checkpoint(WriteRecoveryCheckpointInput {
            project_path: project.project.clone(),
            session_id: session.session_id,
            document_id: "../outside".to_owned(),
            schema_version: 1,
            persisted_updated_at: None,
            content: serde_json::json!({}),
        })
        .expect_err("path-like document IDs must be rejected");

        assert!(matches!(error, RecoveryJournalError::InvalidDocumentId));
    }
}
