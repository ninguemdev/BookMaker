use serde::Serialize;

use crate::application::create_document::{
    create_document as execute_create_document, CreateDocumentError, CreateDocumentInput,
    CreatedDocument,
};
use crate::application::create_project::{
    create_project as execute_create_project, CreateProjectError, CreateProjectInput,
    CreatedProject,
};
use crate::application::move_document::{
    move_document as execute_move_document, MoveDocumentError, MoveDocumentInput, MovedDocument,
};
use crate::application::open_project::{
    open_project as execute_open_project, OpenProjectError, OpenProjectInput, OpenedProject,
};
use crate::application::recent_projects::{
    list_recent_projects as execute_list_recent_projects, record_recent_project, RecentProject,
    RecentProjectUpdate, RecentProjectsError,
};
use crate::application::recovery_journal::{
    begin_recovery_session as execute_begin_recovery_session,
    clear_recovery_checkpoint as execute_clear_recovery_checkpoint,
    end_recovery_session as execute_end_recovery_session,
    list_recovery_checkpoints as execute_list_recovery_checkpoints,
    write_recovery_checkpoint as execute_write_recovery_checkpoint, EndRecoverySessionInput,
    RecoveryCheckpoint, RecoveryDocumentInput, RecoveryJournalError, RecoverySession,
    WriteRecoveryCheckpointInput,
};
use crate::application::rename_document::{
    rename_document as execute_rename_document, RenameDocumentError, RenameDocumentInput,
    RenamedDocument,
};
use tauri::Manager;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    code: &'static str,
    message: &'static str,
}

impl From<&CreateProjectError> for CommandError {
    fn from(error: &CreateProjectError) -> Self {
        Self {
            code: error.code(),
            message: error.user_message(),
        }
    }
}

impl From<&CreateDocumentError> for CommandError {
    fn from(error: &CreateDocumentError) -> Self {
        Self {
            code: error.code(),
            message: error.user_message(),
        }
    }
}

impl From<&RenameDocumentError> for CommandError {
    fn from(error: &RenameDocumentError) -> Self {
        Self {
            code: error.code(),
            message: error.user_message(),
        }
    }
}

impl From<&MoveDocumentError> for CommandError {
    fn from(error: &MoveDocumentError) -> Self {
        Self {
            code: error.code(),
            message: error.user_message(),
        }
    }
}

impl From<&OpenProjectError> for CommandError {
    fn from(error: &OpenProjectError) -> Self {
        Self {
            code: error.code(),
            message: error.user_message(),
        }
    }
}

impl From<&RecentProjectsError> for CommandError {
    fn from(error: &RecentProjectsError) -> Self {
        Self {
            code: error.code(),
            message: error.user_message(),
        }
    }
}

impl From<&RecoveryJournalError> for CommandError {
    fn from(error: &RecoveryJournalError) -> Self {
        Self {
            code: error.code(),
            message: error.user_message(),
        }
    }
}

#[tauri::command]
pub fn create_project(
    app: tauri::AppHandle,
    input: CreateProjectInput,
) -> Result<CreatedProject, CommandError> {
    let created = execute_create_project(input).map_err(|error| {
        log::error!(
            target: "project",
            "create_project_failed code={} error={error}",
            error.code()
        );
        CommandError::from(&error)
    })?;
    record_recent_best_effort(
        &app,
        RecentProjectUpdate {
            project_id: created.project_id.clone(),
            path: created.path.clone(),
            title: created.title.clone(),
        },
    );
    Ok(created)
}

#[tauri::command]
pub fn create_document(input: CreateDocumentInput) -> Result<CreatedDocument, CommandError> {
    execute_create_document(input).map_err(|error| {
        log::error!(
            target: "document",
            "create_document_failed code={} error={error}",
            error.code()
        );
        CommandError::from(&error)
    })
}

#[tauri::command]
pub fn rename_document(input: RenameDocumentInput) -> Result<RenamedDocument, CommandError> {
    execute_rename_document(input).map_err(|error| {
        log::error!(
            target: "document",
            "rename_document_failed code={} error={error}",
            error.code()
        );
        CommandError::from(&error)
    })
}

#[tauri::command]
pub fn move_document(input: MoveDocumentInput) -> Result<MovedDocument, CommandError> {
    execute_move_document(input).map_err(|error| {
        log::error!(
            target: "document",
            "move_document_failed code={} error={error}",
            error.code()
        );
        CommandError::from(&error)
    })
}

#[tauri::command]
pub fn open_project(
    app: tauri::AppHandle,
    input: OpenProjectInput,
) -> Result<OpenedProject, CommandError> {
    let opened = execute_open_project(input).map_err(|error| {
        log::error!(
            target: "project",
            "open_project_failed code={} error={error}",
            error.code()
        );
        CommandError::from(&error)
    })?;
    record_recent_best_effort(
        &app,
        RecentProjectUpdate {
            project_id: opened.project_id.clone(),
            path: opened.path.clone(),
            title: opened.title.clone(),
        },
    );
    Ok(opened)
}

#[tauri::command]
pub fn list_recent_projects(app: tauri::AppHandle) -> Result<Vec<RecentProject>, CommandError> {
    let data_directory = application_data_directory(&app)?;
    execute_list_recent_projects(&data_directory).map_err(|error| {
        log::error!(
            target: "project",
            "list_recent_projects_failed code={} error={error}",
            error.code()
        );
        CommandError::from(&error)
    })
}

#[tauri::command]
pub fn begin_recovery_session(
    project_path: std::path::PathBuf,
) -> Result<RecoverySession, CommandError> {
    execute_begin_recovery_session(&project_path).map_err(map_recovery_error)
}

#[tauri::command]
pub fn end_recovery_session(input: EndRecoverySessionInput) -> Result<(), CommandError> {
    execute_end_recovery_session(input).map_err(map_recovery_error)
}

#[tauri::command]
pub fn write_recovery_checkpoint(
    input: WriteRecoveryCheckpointInput,
) -> Result<RecoveryCheckpoint, CommandError> {
    execute_write_recovery_checkpoint(input).map_err(map_recovery_error)
}

#[tauri::command]
pub fn list_recovery_checkpoints(
    project_path: std::path::PathBuf,
) -> Result<Vec<RecoveryCheckpoint>, CommandError> {
    execute_list_recovery_checkpoints(&project_path).map_err(map_recovery_error)
}

#[tauri::command]
pub fn clear_recovery_checkpoint(input: RecoveryDocumentInput) -> Result<(), CommandError> {
    execute_clear_recovery_checkpoint(input).map_err(map_recovery_error)
}

fn record_recent_best_effort(app: &tauri::AppHandle, project: RecentProjectUpdate) {
    let result = application_data_directory(app).and_then(|data_directory| {
        record_recent_project(&data_directory, project).map_err(|error| CommandError::from(&error))
    });
    if let Err(error) = result {
        log::warn!(
            target: "project",
            "record_recent_project_failed code={}",
            error.code
        );
    }
}

fn application_data_directory(app: &tauri::AppHandle) -> Result<std::path::PathBuf, CommandError> {
    app.path().app_data_dir().map_err(|error| {
        log::error!(target: "project", "resolve_app_data_directory_failed error={error}");
        CommandError {
            code: "recent_projects.unavailable",
            message: "Não foi possível acessar os dados locais do BookMaker.",
        }
    })
}

fn map_recovery_error(error: RecoveryJournalError) -> CommandError {
    log::error!(
        target: "recovery",
        "recovery_operation_failed code={} error={error}",
        error.code()
    );
    CommandError::from(&error)
}
