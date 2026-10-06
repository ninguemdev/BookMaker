use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

pub mod application;
mod commands;
pub mod persistence;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::create_project,
            commands::create_document,
            commands::rename_document,
            commands::move_document,
            commands::trash_document,
            commands::restore_document,
            commands::list_trashed_documents,
            commands::open_project,
            commands::list_recent_projects,
            commands::begin_recovery_session,
            commands::end_recovery_session,
            commands::write_recovery_checkpoint,
            commands::list_recovery_checkpoints,
            commands::clear_recovery_checkpoint
        ])
        .plugin(
            tauri_plugin_log::Builder::new()
                .clear_targets()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                ])
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                .max_file_size(1_000_000)
                .rotation_strategy(RotationStrategy::KeepSome(3))
                .build(),
        )
        .setup(|_| {
            log::info!(target: "native", "application_started");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run BookMaker");
}
