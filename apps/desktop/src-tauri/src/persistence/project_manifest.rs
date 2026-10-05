use serde::{Deserialize, Serialize};

pub(crate) const PROJECT_FORMAT: &str = "bookmaker-project";
pub(crate) const PROJECT_FORMAT_VERSION: i64 = 1;
pub(crate) const PROJECT_CREATOR: &str = "BookMaker";
pub(crate) const PROJECT_EXTENSION: &str = "bookmaker";
pub(crate) const MANIFEST_FILE: &str = "manifest.json";
pub(crate) const DATABASE_FILE: &str = "project.db";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ProjectManifest {
    pub format: String,
    pub format_version: i64,
    pub project_id: String,
    pub created_by: String,
    pub minimum_app_version: String,
}

impl ProjectManifest {
    pub(crate) fn new(project_id: String) -> Self {
        Self {
            format: PROJECT_FORMAT.to_owned(),
            format_version: PROJECT_FORMAT_VERSION,
            project_id,
            created_by: PROJECT_CREATOR.to_owned(),
            minimum_app_version: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }
}
