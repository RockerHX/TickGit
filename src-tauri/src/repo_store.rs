use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use tauri::{AppHandle, Manager, State};

use crate::{
    error::{AppError, AppResult},
    models::{RepositoryConfig, RepositorySummary},
};

pub struct RepositoryStoreState {
    lock: Mutex<()>,
}

impl RepositoryStoreState {
    pub fn new() -> Self {
        Self {
            lock: Mutex::new(()),
        }
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

fn store_path(app: &AppHandle) -> AppResult<PathBuf> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| AppError::new("config_dir_unavailable", error.to_string()))?;

    fs::create_dir_all(&config_dir)
        .map_err(|error| AppError::new("config_dir_create_failed", error.to_string()))?;

    Ok(config_dir.join("repositories.json"))
}

fn read_store(path: &Path) -> AppResult<RepositoryConfig> {
    if !path.exists() {
        return Ok(RepositoryConfig::default());
    }

    let content = fs::read_to_string(path)
        .map_err(|error| AppError::new("store_read_failed", error.to_string()))?;

    serde_json::from_str(&content)
        .map_err(|error| AppError::new("store_parse_failed", error.to_string()))
}

fn write_store(path: &Path, store: &RepositoryConfig) -> AppResult<()> {
    let content = serde_json::to_string_pretty(store)
        .map_err(|error| AppError::new("store_serialize_failed", error.to_string()))?;

    fs::write(path, content).map_err(|error| AppError::new("store_write_failed", error.to_string()))
}

fn repository_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

fn normalize_path(path: &str) -> AppResult<PathBuf> {
    let repository_path = PathBuf::from(path);

    if !repository_path.is_dir() {
        return Err(AppError::new("invalid_repository", "拖入的路径不是文件夹"));
    }

    if !repository_path.join(".git").exists() {
        return Err(AppError::new(
            "invalid_repository",
            "未检测到有效的 Git 仓库",
        ));
    }

    repository_path
        .canonicalize()
        .map_err(|error| AppError::new("invalid_repository", error.to_string()))
}

pub fn list_repositories(
    app: &AppHandle,
    state: State<'_, RepositoryStoreState>,
) -> AppResult<Vec<RepositorySummary>> {
    let _guard = state.lock.lock().expect("repository store poisoned");
    let path = store_path(app)?;
    let mut repositories = read_store(&path)?.repositories;
    repositories.sort_by(|left, right| right.last_opened_at.cmp(&left.last_opened_at));
    Ok(repositories)
}

pub fn add_repository(
    app: &AppHandle,
    state: State<'_, RepositoryStoreState>,
    path: String,
) -> AppResult<RepositorySummary> {
    let _guard = state.lock.lock().expect("repository store poisoned");
    let config_path = store_path(app)?;
    let mut store = read_store(&config_path)?;
    let normalized = normalize_path(&path)?;
    let normalized_str = normalized.to_string_lossy().to_string();

    if store
        .repositories
        .iter()
        .any(|repository| repository.path == normalized_str)
    {
        return Err(AppError::new("repository_exists", "该仓库已存在于列表中"));
    }

    let repository = RepositorySummary {
        name: repository_name(&normalized),
        path: normalized_str.clone(),
        last_opened_at: now_millis(),
    };

    store.repositories.push(repository.clone());
    store.current_path = Some(normalized_str);
    write_store(&config_path, &store)?;

    Ok(repository)
}

pub fn set_current_repository(
    app: &AppHandle,
    state: State<'_, RepositoryStoreState>,
    path: String,
) -> AppResult<()> {
    let _guard = state.lock.lock().expect("repository store poisoned");
    let config_path = store_path(app)?;
    let mut store = read_store(&config_path)?;

    let repository = store
        .repositories
        .iter_mut()
        .find(|repository| repository.path == path)
        .ok_or_else(|| AppError::new("repository_not_found", "仓库不存在"))?;

    repository.last_opened_at = now_millis();
    store.current_path = Some(path);
    write_store(&config_path, &store)
}

pub fn get_current_repository(
    app: &AppHandle,
    state: State<'_, RepositoryStoreState>,
) -> AppResult<Option<RepositorySummary>> {
    let _guard = state.lock.lock().expect("repository store poisoned");
    let config_path = store_path(app)?;
    let store = read_store(&config_path)?;

    Ok(store.current_path.as_ref().and_then(|path| {
        store
            .repositories
            .into_iter()
            .find(|repository| &repository.path == path)
    }))
}
