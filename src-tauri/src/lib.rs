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
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_repositories,
            commands::add_repository,
            commands::set_current_repository,
            commands::get_current_repository,
            commands::get_branch_status,
            commands::get_commit_history,
            commands::get_commit_files,
            commands::get_commit_meta,
            commands::get_commit_file_diff,
            commands::push_current_branch,
            commands::push_to_commit,
            commands::start_push_to_commit,
            commands::start_step_push,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
