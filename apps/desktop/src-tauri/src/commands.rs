use serde::Serialize;

use crate::application::create_project::{
    create_project as execute_create_project, CreateProjectError, CreateProjectInput,
    CreatedProject,
};
use crate::application::open_project::{
    open_project as execute_open_project, OpenProjectError, OpenProjectInput, OpenedProject,
};
use crate::application::recent_projects::{
    list_recent_projects as execute_list_recent_projects, record_recent_project, RecentProject,
    RecentProjectUpdate, RecentProjectsError,
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
