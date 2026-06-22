use std::{fs, path::PathBuf};

use tauri::{AppHandle, Manager, State};

use crate::{
    error::{AppError, AppResult},
    git, jobs,
    models::{
        BranchStatus, CommitDetails, CommitFileChange, CommitFileDiffResult,
        CommitHistoryFilters, CommitHistoryPage, CommitMeta, PushToCommitJobStarted,
        PushToCommitRequest, RepositoryIndex, RepositoryOverview, RepositoryOverviewCacheEntry,
        RepositoryStatusUpdate, RepositorySummary, StepPushJobStarted, StepPushPlan,
        StepPushRequest,
    },
    repo_store::{self, RepositoryStoreState},
};

fn overview_cache_path(app: &AppHandle) -> AppResult<PathBuf> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| AppError::new("config_dir_unavailable", error.to_string()))?;

    fs::create_dir_all(&config_dir)
        .map_err(|error| AppError::new("config_dir_create_failed", error.to_string()))?;

    Ok(config_dir.join("repository-overview-cache.json"))
}

fn read_overview_cache(app: &AppHandle) -> AppResult<Option<RepositoryOverviewCacheEntry>> {
    let path = overview_cache_path(app)?;
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path)
        .map_err(|error| AppError::new("overview_cache_read_failed", error.to_string()))?;
    serde_json::from_str(&content)
        .map(Some)
        .map_err(|error| AppError::new("overview_cache_parse_failed", error.to_string()))
}

fn write_overview_cache(app: &AppHandle, entry: &RepositoryOverviewCacheEntry) -> AppResult<()> {
    let path = overview_cache_path(app)?;
    let content = serde_json::to_string(entry)
        .map_err(|error| AppError::new("overview_cache_serialize_failed", error.to_string()))?;

    fs::write(path, content)
        .map_err(|error| AppError::new("overview_cache_write_failed", error.to_string()))
}

async fn run_blocking<T, F>(task: F) -> AppResult<T>
where
    T: Send + 'static,
    F: FnOnce() -> AppResult<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|error| AppError::new("background_task_failed", error.to_string()))?
}

#[tauri::command]
pub async fn get_repository_index_fast(
    app: AppHandle,
    state: State<'_, RepositoryStoreState>,
) -> AppResult<RepositoryIndex> {
    repo_store::get_repository_index_fast(&app, state)
}

#[tauri::command]
pub async fn get_cached_repository_overview(
    app: AppHandle,
) -> AppResult<Option<RepositoryOverviewCacheEntry>> {
    read_overview_cache(&app)
}

#[tauri::command]
pub async fn refresh_repository_statuses(
    paths: Vec<String>,
) -> AppResult<Vec<RepositoryStatusUpdate>> {
    run_blocking(move || {
        Ok(paths
            .into_iter()
            .map(repo_store::repository_status_update)
            .collect())
    })
    .await
}

#[tauri::command]
pub async fn get_repository_overview(
    app: AppHandle,
    repo_path: String,
    skip: usize,
    limit: usize,
    filters: Option<CommitHistoryFilters>,
) -> AppResult<RepositoryOverview> {
    let overview = run_blocking({
        let repo_path = repo_path.clone();
        let filters = filters.clone();
        move || git::get_repository_overview(&repo_path, skip, limit, filters)
    })
    .await?;

    if skip == 0 {
        let _ = write_overview_cache(
            &app,
            &RepositoryOverviewCacheEntry {
                repo_path,
                skip,
                limit,
                filters,
                overview: overview.clone(),
                cached_at: repo_store::now_millis(),
            },
        );
    }

    Ok(overview)
}

#[tauri::command]
pub async fn get_commit_details(repo_path: String, hash: String) -> AppResult<CommitDetails> {
    run_blocking(move || git::get_commit_details(&repo_path, &hash)).await
}

#[tauri::command]
pub async fn list_repositories(
    app: AppHandle,
    state: State<'_, RepositoryStoreState>,
) -> AppResult<Vec<RepositorySummary>> {
    repo_store::list_repositories(&app, state)
}

#[tauri::command]
pub async fn add_repository(
    app: AppHandle,
    state: State<'_, RepositoryStoreState>,
    path: String,
) -> AppResult<RepositorySummary> {
    repo_store::add_repository(&app, state, path)
}

#[tauri::command]
pub async fn set_current_repository(
    app: AppHandle,
    state: State<'_, RepositoryStoreState>,
    path: String,
) -> AppResult<()> {
    repo_store::set_current_repository(&app, state, path)
}

#[tauri::command]
pub async fn get_current_repository(
    app: AppHandle,
    state: State<'_, RepositoryStoreState>,
) -> AppResult<Option<RepositorySummary>> {
    repo_store::get_current_repository(&app, state)
}

#[tauri::command]
pub async fn remove_repository(
    app: AppHandle,
    state: State<'_, RepositoryStoreState>,
    path: String,
) -> AppResult<Option<RepositorySummary>> {
    repo_store::remove_repository(&app, state, path)
}

#[tauri::command]
pub async fn relocate_repository(
    app: AppHandle,
    state: State<'_, RepositoryStoreState>,
    old_path: String,
    new_path: String,
) -> AppResult<RepositorySummary> {
    repo_store::relocate_repository(&app, state, old_path, new_path)
}

#[tauri::command]
pub async fn get_branch_status(repo_path: String) -> AppResult<BranchStatus> {
    run_blocking(move || git::get_branch_status(&repo_path)).await
}

#[tauri::command]
pub async fn refresh_remote_tracking(repo_path: String) -> AppResult<()> {
    run_blocking(move || git::refresh_remote_tracking(&repo_path)).await
}

#[tauri::command]
pub async fn list_local_branches(repo_path: String) -> AppResult<Vec<String>> {
    run_blocking(move || git::list_local_branches(&repo_path)).await
}

#[tauri::command]
pub async fn checkout_branch(repo_path: String, branch: String) -> AppResult<()> {
    run_blocking(move || git::checkout_branch(&repo_path, &branch)).await
}

#[tauri::command]
pub async fn get_commit_history(
    repo_path: String,
    skip: usize,
    limit: usize,
    filters: Option<CommitHistoryFilters>,
) -> AppResult<CommitHistoryPage> {
    run_blocking(move || git::get_commit_history(&repo_path, skip, limit, filters)).await
}

#[tauri::command]
pub async fn get_commit_files(repo_path: String, hash: String) -> AppResult<Vec<CommitFileChange>> {
    run_blocking(move || git::get_commit_files(&repo_path, &hash)).await
}

#[tauri::command]
pub async fn get_commit_meta(repo_path: String, hash: String) -> AppResult<CommitMeta> {
    run_blocking(move || git::get_commit_meta(&repo_path, &hash)).await
}

#[tauri::command]
pub async fn get_commit_file_diff(
    repo_path: String,
    hash: String,
    file_path: String,
    previous_path: Option<String>,
    ignore_whitespace: bool,
) -> AppResult<CommitFileDiffResult> {
    run_blocking(move || {
        git::get_commit_file_diff(
            &repo_path,
            &hash,
            &file_path,
            previous_path.as_deref(),
            ignore_whitespace,
        )
    })
    .await
}

#[tauri::command]
pub async fn start_push_current_branch(
    app: AppHandle,
    jobs: State<'_, jobs::PushToCommitManager>,
    gate: State<'_, jobs::PushExecutionGate>,
    repo_path: String,
    branch: String,
) -> AppResult<PushToCommitJobStarted> {
    jobs::start_push_current_branch(app, jobs, gate, repo_path, branch)
}

#[tauri::command]
pub async fn save_window_size(
    app: AppHandle,
    state: State<'_, RepositoryStoreState>,
    width: f64,
    height: f64,
) -> AppResult<()> {
    repo_store::save_window_size(&app, state, width, height)
}

#[tauri::command]
pub async fn get_step_push_plan(repo_path: String, target_hash: String) -> AppResult<StepPushPlan> {
    run_blocking(move || git::get_step_push_plan(&repo_path, &target_hash)).await
}

#[tauri::command]
pub async fn start_push_to_commit(
    app: AppHandle,
    jobs: State<'_, jobs::PushToCommitManager>,
    gate: State<'_, jobs::PushExecutionGate>,
    request: PushToCommitRequest,
) -> AppResult<PushToCommitJobStarted> {
    jobs::start_push_to_commit(app, jobs, gate, request)
}

#[tauri::command]
pub async fn start_step_push(
    app: AppHandle,
    jobs: State<'_, jobs::StepPushManager>,
    gate: State<'_, jobs::PushExecutionGate>,
    request: StepPushRequest,
) -> AppResult<StepPushJobStarted> {
    jobs::start_step_push(app, jobs, gate, request)
}
