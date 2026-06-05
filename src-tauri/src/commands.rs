use tauri::{AppHandle, State};

use crate::{
    error::AppResult,
    git, jobs,
    models::{
        BranchStatus, CommitCreated, CommitFileChange, CommitFileDiffResult,
        CommitHistoryFilters, CommitHistoryPage, CommitMeta, PushToCommitJobStarted,
        PushToCommitRequest, RepositorySummary, StepPushJobStarted, StepPushPlan, StepPushRequest,
        WorkspaceChangeSection, WorkspaceStatus,
    },
    repo_store::{self, RepositoryStoreState},
};

#[tauri::command]
pub fn list_repositories(
    app: AppHandle,
    state: State<'_, RepositoryStoreState>,
) -> AppResult<Vec<RepositorySummary>> {
    repo_store::list_repositories(&app, state)
}

#[tauri::command]
pub fn add_repository(
    app: AppHandle,
    state: State<'_, RepositoryStoreState>,
    path: String,
) -> AppResult<RepositorySummary> {
    repo_store::add_repository(&app, state, path)
}

#[tauri::command]
pub fn set_current_repository(
    app: AppHandle,
    state: State<'_, RepositoryStoreState>,
    path: String,
) -> AppResult<()> {
    repo_store::set_current_repository(&app, state, path)
}

#[tauri::command]
pub fn get_current_repository(
    app: AppHandle,
    state: State<'_, RepositoryStoreState>,
) -> AppResult<Option<RepositorySummary>> {
    repo_store::get_current_repository(&app, state)
}

#[tauri::command]
pub fn remove_repository(
    app: AppHandle,
    state: State<'_, RepositoryStoreState>,
    path: String,
) -> AppResult<Option<RepositorySummary>> {
    repo_store::remove_repository(&app, state, path)
}

#[tauri::command]
pub fn relocate_repository(
    app: AppHandle,
    state: State<'_, RepositoryStoreState>,
    old_path: String,
    new_path: String,
) -> AppResult<RepositorySummary> {
    repo_store::relocate_repository(&app, state, old_path, new_path)
}

#[tauri::command]
pub fn get_branch_status(repo_path: String) -> AppResult<BranchStatus> {
    git::get_branch_status(&repo_path)
}

#[tauri::command]
pub fn refresh_remote_tracking(repo_path: String) -> AppResult<()> {
    git::refresh_remote_tracking(&repo_path)
}

#[tauri::command]
pub fn list_local_branches(repo_path: String) -> AppResult<Vec<String>> {
    git::list_local_branches(&repo_path)
}

#[tauri::command]
pub fn checkout_branch(repo_path: String, branch: String) -> AppResult<()> {
    git::checkout_branch(&repo_path, &branch)
}

#[tauri::command]
pub fn get_commit_history(
    repo_path: String,
    skip: usize,
    limit: usize,
    filters: Option<CommitHistoryFilters>,
) -> AppResult<CommitHistoryPage> {
    git::get_commit_history(&repo_path, skip, limit, filters)
}

#[tauri::command]
pub fn get_commit_files(repo_path: String, hash: String) -> AppResult<Vec<CommitFileChange>> {
    git::get_commit_files(&repo_path, &hash)
}

#[tauri::command]
pub fn get_commit_meta(repo_path: String, hash: String) -> AppResult<CommitMeta> {
    git::get_commit_meta(&repo_path, &hash)
}

#[tauri::command]
pub fn get_commit_file_diff(
    repo_path: String,
    hash: String,
    file_path: String,
    previous_path: Option<String>,
    ignore_whitespace: bool,
) -> AppResult<CommitFileDiffResult> {
    git::get_commit_file_diff(
        &repo_path,
        &hash,
        &file_path,
        previous_path.as_deref(),
        ignore_whitespace,
    )
}

#[tauri::command]
pub fn get_workspace_status(repo_path: String) -> AppResult<WorkspaceStatus> {
    git::get_workspace_status(&repo_path)
}

#[tauri::command]
pub fn get_workspace_file_diff(
    repo_path: String,
    section: WorkspaceChangeSection,
    file_path: String,
    previous_path: Option<String>,
    ignore_whitespace: bool,
) -> AppResult<CommitFileDiffResult> {
    git::get_workspace_file_diff(
        &repo_path,
        section,
        &file_path,
        previous_path.as_deref(),
        ignore_whitespace,
    )
}

#[tauri::command]
pub fn stage_workspace_file(repo_path: String, file_path: String) -> AppResult<()> {
    git::stage_workspace_file(&repo_path, &file_path)
}

#[tauri::command]
pub fn unstage_workspace_file(repo_path: String, file_path: String) -> AppResult<()> {
    git::unstage_workspace_file(&repo_path, &file_path)
}

#[tauri::command]
pub fn create_commit(repo_path: String, message: String) -> AppResult<CommitCreated> {
    git::create_commit(&repo_path, &message)
}

#[tauri::command]
pub fn push_current_branch(repo_path: String) -> AppResult<()> {
    git::push_current_branch(&repo_path)
}

#[tauri::command]
pub fn start_push_current_branch(
    app: AppHandle,
    jobs: State<'_, jobs::PushToCommitManager>,
    gate: State<'_, jobs::PushExecutionGate>,
    repo_path: String,
    branch: String,
) -> AppResult<PushToCommitJobStarted> {
    jobs::start_push_current_branch(app, jobs, gate, repo_path, branch)
}

#[tauri::command]
pub fn save_window_size(
    app: AppHandle,
    state: State<'_, RepositoryStoreState>,
    width: f64,
    height: f64,
) -> AppResult<()> {
    repo_store::save_window_size(&app, state, width, height)
}

#[tauri::command]
pub fn push_to_commit(repo_path: String, branch: String, hash: String) -> AppResult<()> {
    git::push_to_commit(&repo_path, &branch, &hash)
}

#[tauri::command]
pub fn get_step_push_plan(repo_path: String, target_hash: String) -> AppResult<StepPushPlan> {
    git::get_step_push_plan(&repo_path, &target_hash)
}

#[tauri::command]
pub fn start_push_to_commit(
    app: AppHandle,
    jobs: State<'_, jobs::PushToCommitManager>,
    gate: State<'_, jobs::PushExecutionGate>,
    request: PushToCommitRequest,
) -> AppResult<PushToCommitJobStarted> {
    jobs::start_push_to_commit(app, jobs, gate, request)
}

#[tauri::command]
pub fn start_step_push(
    app: AppHandle,
    jobs: State<'_, jobs::StepPushManager>,
    gate: State<'_, jobs::PushExecutionGate>,
    request: StepPushRequest,
) -> AppResult<StepPushJobStarted> {
    jobs::start_step_push(app, jobs, gate, request)
}
