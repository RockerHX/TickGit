mod commands;
mod error;
mod git;
mod jobs;
mod models;
mod repo_store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(repo_store::RepositoryStoreState::new())
        .manage(jobs::PushToCommitManager::new())
        .manage(jobs::StepPushManager::new())
        .manage(jobs::PushExecutionGate::new())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if let Err(error) = repo_store::apply_initial_window_size(app.handle()) {
                eprintln!("failed to apply initial window size: {}", error.message);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_repositories,
            commands::add_repository,
            commands::set_current_repository,
            commands::get_current_repository,
            commands::get_branch_status,
            commands::refresh_remote_tracking,
            commands::list_local_branches,
            commands::checkout_branch,
            commands::get_commit_history,
            commands::get_commit_files,
            commands::get_commit_meta,
            commands::get_commit_file_diff,
            commands::push_current_branch,
            commands::start_push_current_branch,
            commands::save_window_size,
            commands::push_to_commit,
            commands::get_step_push_plan,
            commands::start_push_to_commit,
            commands::start_step_push,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
