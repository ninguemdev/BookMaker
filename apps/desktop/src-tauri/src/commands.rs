use serde::Serialize;

use crate::application::create_project::{
    create_project as execute_create_project, CreateProjectError, CreateProjectInput,
    CreatedProject,
};
use crate::application::open_project::{
    open_project as execute_open_project, OpenProjectError, OpenProjectInput, OpenedProject,
};

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

#[tauri::command]
pub fn create_project(input: CreateProjectInput) -> Result<CreatedProject, CommandError> {
    execute_create_project(input).map_err(|error| {
        log::error!(
            target: "project",
            "create_project_failed code={} error={error}",
            error.code()
        );
        CommandError::from(&error)
    })
}

#[tauri::command]
pub fn open_project(input: OpenProjectInput) -> Result<OpenedProject, CommandError> {
    execute_open_project(input).map_err(|error| {
        log::error!(
            target: "project",
            "open_project_failed code={} error={error}",
            error.code()
        );
        CommandError::from(&error)
    })
}
