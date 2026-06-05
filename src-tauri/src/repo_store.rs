use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use tauri::{AppHandle, Manager, PhysicalSize, Size, State, WebviewWindow};

use crate::{
    error::{AppError, AppResult},
    git,
    models::{
        RepositoryConfig, RepositoryStatus, RepositorySummary, StoredRepository, WindowSizeConfig,
    },
};

const DEFAULT_WINDOW_WIDTH_RATIO: f64 = 0.75;
const DEFAULT_WINDOW_HEIGHT_RATIO: f64 = 0.75;
const MIN_WINDOW_WIDTH: f64 = 720.0;
const MIN_WINDOW_HEIGHT: f64 = 480.0;
const MAX_SAVED_WINDOW_RATIO_BEFORE_RESET: f64 = 0.95;

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

fn sort_repositories(repositories: &mut [StoredRepository]) {
    repositories.sort_by(|left, right| {
        right
            .last_opened_at
            .cmp(&left.last_opened_at)
            .then_with(|| left.path.cmp(&right.path))
    });
}

fn find_current_repository(store: &RepositoryConfig) -> Option<RepositorySummary> {
    store.current_path.as_ref().and_then(|path| {
        store
            .repositories
            .iter()
            .find(|repository| &repository.path == path)
            .map(repository_summary)
    })
}

fn repository_status(path: &str) -> (RepositoryStatus, Option<String>, Option<String>) {
    let path = Path::new(path);

    if !path.exists() {
        return (
            RepositoryStatus::Missing,
            Some("仓库路径不存在".to_string()),
            Some("repository_missing".to_string()),
        );
    }

    match git::resolve_repository_path(path.to_string_lossy().as_ref()) {
        Ok(_) => (RepositoryStatus::Available, None, None),
        Err(error) => (RepositoryStatus::Invalid, Some(error.message), Some(error.code)),
    }
}

fn repository_summary(repository: &StoredRepository) -> RepositorySummary {
    let (status, disabled_reason, disabled_reason_code) = repository_status(&repository.path);

    RepositorySummary {
        name: repository.name.clone(),
        path: repository.path.clone(),
        last_opened_at: repository.last_opened_at,
        status,
        disabled_reason,
        disabled_reason_code,
    }
}

fn add_repository_to_store(
    store: &mut RepositoryConfig,
    repository_path: &Path,
    opened_at: i64,
) -> AppResult<RepositorySummary> {
    let normalized_path = repository_path.to_string_lossy().to_string();

    if store
        .repositories
        .iter()
        .any(|repository| repository.path == normalized_path)
    {
        return Err(AppError::new("repository_exists", "该仓库已存在于列表中"));
    }

    let repository = StoredRepository {
        name: repository_name(repository_path),
        path: normalized_path.clone(),
        last_opened_at: opened_at,
    };

    store.repositories.push(repository.clone());
    store.current_path = Some(normalized_path);

    Ok(repository_summary(&repository))
}

fn normalize_repository_store_path(path: &str) -> AppResult<String> {
    Ok(git::resolve_repository_path(path)?
        .to_string_lossy()
        .to_string())
}

fn set_current_repository_in_store(
    store: &mut RepositoryConfig,
    path: &str,
    opened_at: i64,
) -> AppResult<()> {
    let repository = store
        .repositories
        .iter_mut()
        .find(|repository| repository.path == path)
        .ok_or_else(|| AppError::new("repository_not_found", "仓库不存在"))?;

    repository.last_opened_at = opened_at;
    store.current_path = Some(path.to_string());

    Ok(())
}

fn remove_repository_from_store(
    store: &mut RepositoryConfig,
    path: &str,
) -> AppResult<Option<RepositorySummary>> {
    let position = store
        .repositories
        .iter()
        .position(|repository| repository.path == path)
        .ok_or_else(|| AppError::new("repository_not_found", "仓库不存在"))?;

    store.repositories.remove(position);

    if store.current_path.as_deref() == Some(path) {
        sort_repositories(&mut store.repositories);
        store.current_path = store
            .repositories
            .first()
            .map(|repository| repository.path.clone());
    }

    Ok(find_current_repository(store))
}

fn relocate_repository_in_store(
    store: &mut RepositoryConfig,
    old_path: &str,
    new_path: &Path,
    opened_at: i64,
) -> AppResult<RepositorySummary> {
    let normalized_new_path = new_path.to_string_lossy().to_string();

    if store
        .repositories
        .iter()
        .any(|repository| repository.path == normalized_new_path && repository.path != old_path)
    {
        return Err(AppError::new("repository_exists", "该仓库已存在于列表中"));
    }

    let repository = store
        .repositories
        .iter_mut()
        .find(|repository| repository.path == old_path)
        .ok_or_else(|| AppError::new("repository_not_found", "仓库不存在"))?;

    repository.name = repository_name(new_path);
    repository.path = normalized_new_path.clone();
    repository.last_opened_at = opened_at;
    store.current_path = Some(normalized_new_path);

    Ok(repository_summary(repository))
}

fn sanitize_window_size(
    width: f64,
    height: f64,
    max_width: Option<f64>,
    max_height: Option<f64>,
) -> Option<WindowSizeConfig> {
    if !width.is_finite() || !height.is_finite() {
        return None;
    }

    let width = match max_width {
        Some(limit) => width.min(limit),
        None => width,
    };
    let height = match max_height {
        Some(limit) => height.min(limit),
        None => height,
    };

    Some(WindowSizeConfig {
        width: width.max(MIN_WINDOW_WIDTH).round(),
        height: height.max(MIN_WINDOW_HEIGHT).round(),
    })
}

fn apply_window_size(window: &WebviewWindow, size: &WindowSizeConfig) -> AppResult<()> {
    window
        .set_size(Size::Physical(PhysicalSize::new(
            size.width as u32,
            size.height as u32,
        )))
        .map_err(|error| AppError::new("window_resize_failed", error.to_string()))
}

fn build_default_window_size(window: &WebviewWindow) -> AppResult<WindowSizeConfig> {
    let monitor = window
        .current_monitor()
        .map_err(|error| AppError::new("monitor_lookup_failed", error.to_string()))?
        .ok_or_else(|| AppError::new("monitor_unavailable", "无法获取当前屏幕尺寸"))?;

    let size = monitor.size();

    sanitize_window_size(
        f64::from(size.width) * DEFAULT_WINDOW_WIDTH_RATIO,
        f64::from(size.height) * DEFAULT_WINDOW_HEIGHT_RATIO,
        Some(f64::from(size.width)),
        Some(f64::from(size.height)),
    )
    .ok_or_else(|| AppError::new("window_size_invalid", "默认窗口尺寸无效"))
}

fn should_reset_saved_window_size(
    saved_size: &WindowSizeConfig,
    monitor_width: f64,
    monitor_height: f64,
) -> bool {
    saved_size.width >= monitor_width * MAX_SAVED_WINDOW_RATIO_BEFORE_RESET
        || saved_size.height >= monitor_height * MAX_SAVED_WINDOW_RATIO_BEFORE_RESET
}

pub fn apply_initial_window_size(app: &AppHandle) -> AppResult<()> {
    let config_path = store_path(app)?;
    let store = read_store(&config_path)?;
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::new("window_not_found", "未找到主窗口"))?;

    let monitor = window
        .current_monitor()
        .map_err(|error| AppError::new("monitor_lookup_failed", error.to_string()))?
        .ok_or_else(|| AppError::new("monitor_unavailable", "无法获取当前屏幕尺寸"))?;
    let monitor_size = monitor.size();

    let monitor_width = f64::from(monitor_size.width);
    let monitor_height = f64::from(monitor_size.height);

    let size = match store.window_size {
        Some(saved_size)
            if !should_reset_saved_window_size(&saved_size, monitor_width, monitor_height) =>
        {
            sanitize_window_size(
                saved_size.width,
                saved_size.height,
                Some(monitor_width),
                Some(monitor_height),
            )
            .ok_or_else(|| AppError::new("window_size_invalid", "保存的窗口尺寸无效"))?
        }
        _ => build_default_window_size(&window)?,
    };

    apply_window_size(&window, &size)?;
    window
        .center()
        .map_err(|error| AppError::new("window_center_failed", error.to_string()))?;

    Ok(())
}

pub fn save_window_size(
    app: &AppHandle,
    state: State<'_, RepositoryStoreState>,
    width: f64,
    height: f64,
) -> AppResult<()> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::new("window_not_found", "未找到主窗口"))?;
    let monitor = window
        .current_monitor()
        .map_err(|error| AppError::new("monitor_lookup_failed", error.to_string()))?
        .ok_or_else(|| AppError::new("monitor_unavailable", "无法获取当前屏幕尺寸"))?;
    let monitor_size = monitor.size();

    let window_size = sanitize_window_size(
        width,
        height,
        Some(f64::from(monitor_size.width)),
        Some(f64::from(monitor_size.height)),
    )
    .ok_or_else(|| AppError::new("window_size_invalid", "窗口尺寸无效"))?;

    let _guard = state.lock.lock().expect("repository store poisoned");
    let config_path = store_path(app)?;
    let mut store = read_store(&config_path)?;
    store.window_size = Some(window_size);
    write_store(&config_path, &store)
}

pub fn list_repositories(
    app: &AppHandle,
    state: State<'_, RepositoryStoreState>,
) -> AppResult<Vec<RepositorySummary>> {
    let _guard = state.lock.lock().expect("repository store poisoned");
    let path = store_path(app)?;
    let mut repositories = read_store(&path)?.repositories;
    sort_repositories(&mut repositories);
    Ok(repositories.iter().map(repository_summary).collect())
}

pub fn add_repository(
    app: &AppHandle,
    state: State<'_, RepositoryStoreState>,
    path: String,
) -> AppResult<RepositorySummary> {
    let _guard = state.lock.lock().expect("repository store poisoned");
    let config_path = store_path(app)?;
    let mut store = read_store(&config_path)?;
    let normalized_path = normalize_repository_store_path(&path)?;
    let repository =
        add_repository_to_store(&mut store, Path::new(&normalized_path), now_millis())?;
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
    set_current_repository_in_store(&mut store, &path, now_millis())?;
    write_store(&config_path, &store)
}

pub fn get_current_repository(
    app: &AppHandle,
    state: State<'_, RepositoryStoreState>,
) -> AppResult<Option<RepositorySummary>> {
    let _guard = state.lock.lock().expect("repository store poisoned");
    let config_path = store_path(app)?;
    let store = read_store(&config_path)?;
    Ok(find_current_repository(&store))
}

pub fn remove_repository(
    app: &AppHandle,
    state: State<'_, RepositoryStoreState>,
    path: String,
) -> AppResult<Option<RepositorySummary>> {
    let _guard = state.lock.lock().expect("repository store poisoned");
    let config_path = store_path(app)?;
    let mut store = read_store(&config_path)?;
    let current_repository = remove_repository_from_store(&mut store, &path)?;
    write_store(&config_path, &store)?;
    Ok(current_repository)
}

pub fn relocate_repository(
    app: &AppHandle,
    state: State<'_, RepositoryStoreState>,
    old_path: String,
    new_path: String,
) -> AppResult<RepositorySummary> {
    let _guard = state.lock.lock().expect("repository store poisoned");
    let config_path = store_path(app)?;
    let mut store = read_store(&config_path)?;
    let normalized_new_path = normalize_repository_store_path(&new_path)?;
    let repository = relocate_repository_in_store(
        &mut store,
        &old_path,
        Path::new(&normalized_new_path),
        now_millis(),
    )?;
    write_store(&config_path, &store)?;
    Ok(repository)
}

#[cfg(test)]
mod tests {
    use super::{
        add_repository_to_store, find_current_repository, normalize_repository_store_path,
        read_store, relocate_repository_in_store, remove_repository_from_store, repository_summary,
        set_current_repository_in_store, sort_repositories, write_store,
    };
    use crate::models::{RepositoryConfig, RepositoryStatus, StoredRepository, WindowSizeConfig};
    use std::{
        env, fs,
        path::{Path, PathBuf},
        process::Command,
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static NEXT_TEST_ID: AtomicUsize = AtomicUsize::new(1);

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(prefix: &str) -> Self {
            let suffix = NEXT_TEST_ID.fetch_add(1, Ordering::SeqCst);
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default();
            let path = env::temp_dir().join(format!(
                "tickgit-{prefix}-{}-{timestamp}-{suffix}",
                std::process::id()
            ));

            fs::create_dir_all(&path).expect("create temp test directory");
            Self { path }
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn run_git(path: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .output()
            .expect("run git command");

        if !output.status.success() {
            panic!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    fn init_repo() -> TestDir {
        let repo = TestDir::new("repo-store");
        run_git(&repo.path, &["init"]);
        run_git(&repo.path, &["config", "user.name", "TickGit Tests"]);
        run_git(
            &repo.path,
            &["config", "user.email", "tickgit-tests@example.com"],
        );
        repo
    }

    fn repository(path: &str, last_opened_at: i64) -> StoredRepository {
        StoredRepository {
            name: Path::new(path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            path: path.to_string(),
            last_opened_at,
        }
    }

    #[test]
    fn reads_and_writes_store_file() {
        let temp = TestDir::new("store-file");
        let store_path = temp.path.join("repositories.json");
        let store = RepositoryConfig {
            repositories: vec![repository("/tmp/repo-a", 20)],
            current_path: Some("/tmp/repo-a".to_string()),
            window_size: Some(WindowSizeConfig {
                width: 960.0,
                height: 540.0,
            }),
        };

        write_store(&store_path, &store).unwrap();
        let loaded = read_store(&store_path).unwrap();

        assert_eq!(loaded.repositories.len(), 1);
        assert_eq!(loaded.current_path.as_deref(), Some("/tmp/repo-a"));
        assert_eq!(
            loaded.window_size.as_ref().map(|size| size.width),
            Some(960.0)
        );
    }

    #[test]
    fn sorts_repositories_by_last_opened_at_descending() {
        let mut repositories = vec![
            repository("/tmp/repo-a", 10),
            repository("/tmp/repo-b", 30),
            repository("/tmp/repo-c", 20),
        ];

        sort_repositories(&mut repositories);

        let paths = repositories
            .iter()
            .map(|repository| repository.path.as_str())
            .collect::<Vec<_>>();

        assert_eq!(paths, vec!["/tmp/repo-b", "/tmp/repo-c", "/tmp/repo-a"]);
    }

    #[test]
    fn sorts_repositories_stably_when_opened_at_matches() {
        let mut repositories = vec![
            repository("/tmp/repo-c", 10),
            repository("/tmp/repo-a", 10),
            repository("/tmp/repo-b", 10),
        ];

        sort_repositories(&mut repositories);

        let paths = repositories
            .iter()
            .map(|repository| repository.path.as_str())
            .collect::<Vec<_>>();

        assert_eq!(paths, vec!["/tmp/repo-a", "/tmp/repo-b", "/tmp/repo-c"]);
    }

    #[test]
    fn marks_missing_and_invalid_repository_summaries() {
        let missing = repository("/tmp/tickgit-missing-repo-summary", 10);
        let plain_dir = TestDir::new("plain-repo-summary");

        let missing_summary = repository_summary(&missing);
        let invalid_summary =
            repository_summary(&repository(plain_dir.path.to_string_lossy().as_ref(), 20));

        assert_eq!(missing_summary.status, RepositoryStatus::Missing);
        assert_eq!(
            missing_summary.disabled_reason.as_deref(),
            Some("仓库路径不存在")
        );
        assert_eq!(invalid_summary.status, RepositoryStatus::Invalid);
        assert_eq!(
            invalid_summary.disabled_reason.as_deref(),
            Some("当前目录不是 Git 仓库")
        );
    }

    #[test]
    fn rejects_duplicate_repository_paths() {
        let repo = init_repo();
        let mut store = RepositoryConfig::default();
        let opened_at = 100;

        add_repository_to_store(&mut store, &repo.path, opened_at).unwrap();
        let error = add_repository_to_store(&mut store, &repo.path, opened_at).unwrap_err();

        assert_eq!(error.code, "repository_exists");
        assert_eq!(error.message, "该仓库已存在于列表中");
    }

    #[test]
    fn normalizes_repository_store_path_before_lookup() {
        let repo = init_repo();
        let repo_alias = repo.path.join(".");

        let normalized =
            normalize_repository_store_path(repo_alias.to_string_lossy().as_ref()).unwrap();

        assert_eq!(
            normalized,
            repo.path.canonicalize().unwrap().to_string_lossy()
        );
    }

    #[test]
    fn tracks_current_repository_after_add_and_set() {
        let repo_a = init_repo();
        let repo_b = init_repo();
        let mut store = RepositoryConfig::default();

        let repo_a_summary = add_repository_to_store(&mut store, &repo_a.path, 10).unwrap();
        let repo_b_summary = add_repository_to_store(&mut store, &repo_b.path, 20).unwrap();

        assert_eq!(
            find_current_repository(&store).map(|repository| repository.path),
            Some(repo_b_summary.path.clone())
        );

        set_current_repository_in_store(&mut store, &repo_a_summary.path, 30).unwrap();

        let current = find_current_repository(&store).unwrap();
        assert_eq!(current.path, repo_a_summary.path);
        assert_eq!(current.last_opened_at, 30);
        assert_eq!(current.status, RepositoryStatus::Available);
    }

    #[test]
    fn returns_none_when_current_repository_missing() {
        let store = RepositoryConfig {
            repositories: vec![repository("/tmp/repo-a", 10)],
            current_path: Some("/tmp/missing".to_string()),
            window_size: None,
        };

        assert!(find_current_repository(&store).is_none());
    }

    #[test]
    fn rejects_setting_unknown_current_repository() {
        let mut store = RepositoryConfig {
            repositories: vec![repository("/tmp/repo-a", 10)],
            current_path: None,
            window_size: None,
        };

        let error = set_current_repository_in_store(&mut store, "/tmp/repo-b", 20).unwrap_err();

        assert_eq!(error.code, "repository_not_found");
        assert_eq!(error.message, "仓库不存在");
    }

    #[test]
    fn removes_current_repository_and_selects_most_recent_remaining() {
        let mut store = RepositoryConfig {
            repositories: vec![
                repository("/tmp/repo-a", 10),
                repository("/tmp/repo-b", 30),
                repository("/tmp/repo-c", 20),
            ],
            current_path: Some("/tmp/repo-b".to_string()),
            window_size: None,
        };

        let current = remove_repository_from_store(&mut store, "/tmp/repo-b").unwrap();

        assert_eq!(store.repositories.len(), 2);
        assert_eq!(store.current_path.as_deref(), Some("/tmp/repo-c"));
        assert_eq!(
            current.map(|repository| repository.path),
            Some("/tmp/repo-c".to_string())
        );
    }

    #[test]
    fn removes_non_current_repository_without_changing_current() {
        let mut store = RepositoryConfig {
            repositories: vec![repository("/tmp/repo-a", 10), repository("/tmp/repo-b", 20)],
            current_path: Some("/tmp/repo-a".to_string()),
            window_size: None,
        };

        let current = remove_repository_from_store(&mut store, "/tmp/repo-b").unwrap();

        assert_eq!(store.repositories.len(), 1);
        assert_eq!(store.current_path.as_deref(), Some("/tmp/repo-a"));
        assert_eq!(
            current.map(|repository| repository.path),
            Some("/tmp/repo-a".to_string())
        );
    }

    #[test]
    fn clears_current_repository_when_removing_last_repository() {
        let mut store = RepositoryConfig {
            repositories: vec![repository("/tmp/repo-a", 10)],
            current_path: Some("/tmp/repo-a".to_string()),
            window_size: None,
        };

        let current = remove_repository_from_store(&mut store, "/tmp/repo-a").unwrap();

        assert!(store.repositories.is_empty());
        assert_eq!(store.current_path, None);
        assert!(current.is_none());
    }

    #[test]
    fn relocates_repository_and_selects_new_path() {
        let old_path = "/tmp/moved-repo";
        let new_repo = init_repo();
        let mut store = RepositoryConfig {
            repositories: vec![repository(old_path, 10)],
            current_path: Some(old_path.to_string()),
            window_size: None,
        };

        let relocated =
            relocate_repository_in_store(&mut store, old_path, &new_repo.path, 40).unwrap();

        assert_eq!(relocated.path, new_repo.path.to_string_lossy());
        assert_eq!(
            relocated.name,
            new_repo.path.file_name().unwrap().to_string_lossy()
        );
        assert_eq!(relocated.last_opened_at, 40);
        assert_eq!(relocated.status, RepositoryStatus::Available);
        assert_eq!(store.current_path.as_deref(), Some(relocated.path.as_str()));
    }

    #[test]
    fn rejects_relocating_repository_to_duplicate_path() {
        let repo_a = init_repo();
        let repo_b = init_repo();
        let mut store = RepositoryConfig {
            repositories: vec![
                repository(repo_a.path.to_string_lossy().as_ref(), 10),
                repository(repo_b.path.to_string_lossy().as_ref(), 20),
            ],
            current_path: Some(repo_a.path.to_string_lossy().to_string()),
            window_size: None,
        };

        let error = relocate_repository_in_store(
            &mut store,
            repo_a.path.to_string_lossy().as_ref(),
            &repo_b.path,
            30,
        )
        .unwrap_err();

        assert_eq!(error.code, "repository_exists");
        assert_eq!(error.message, "该仓库已存在于列表中");
    }
}
