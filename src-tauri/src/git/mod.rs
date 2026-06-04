use std::{collections::HashSet, path::Path, process::Command};

mod command;
mod history;
mod parse;
mod repository;

use command::{git_run, git_text, git_trimmed};
pub use history::{get_commit_files, get_commit_history, get_commit_meta};
use repository::{
    branch_status_for_path, current_branch_matching, current_branch_name, sync_origin_tracking,
};
pub use repository::{
    checkout_branch, get_branch_status, list_local_branches, refresh_remote_tracking,
    resolve_repository_path, validate_current_branch,
};

use crate::error::{AppError, AppResult};

const REMOTE_NAME: &str = "origin";
// Git 约定的空树对象。初始提交没有 parent 时，使用它与目标提交做 diff，
// 才能和普通提交一样统一走 diff 参数（例如 -w 忽略空白）逻辑。
const EMPTY_TREE_HASH: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const UNSAFE_PUSH_TARGET_MESSAGE: &str =
    "该 Commit 未推送，但不在 first-parent 安全路径上，不能作为 step push / push to commit 目标";
const BRANCH_BEHIND_REMOTE_MESSAGE: &str =
    "远端已有更新，TickGit 暂不能安全推送。请先使用 GitHub Desktop 或 SourceTree 同步远端后，再回到 TickGit 刷新重试。";
const BRANCH_MISMATCH_MESSAGE: &str = "目标分支与当前检出分支不一致，已拒绝推送";

fn is_ancestor(repo_path: &Path, ancestor: &str, descendant: &str) -> AppResult<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat")
        .env("PAGER", "cat")
        .output()
        .map_err(|error| AppError::new("git_unavailable", error.to_string()))?;

    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let message = if !stderr.is_empty() { stderr } else { stdout };

            Err(AppError::new(
                "git_command_failed",
                if message.is_empty() {
                    "Git 命令执行失败".to_string()
                } else {
                    message
                },
            ))
        }
    }
}

fn ensure_remote_fast_forward_target(
    repo_path: &Path,
    branch: &str,
    target_hash: &str,
) -> AppResult<()> {
    let refspec = format!("{target_hash}:refs/heads/{branch}");

    match git_run(repo_path, &["push", "--dry-run", REMOTE_NAME, &refspec]) {
        Ok(()) => Ok(()),
        Err(error)
            if error.code == "git_command_failed"
                && is_remote_outdated_push_error(&error.message) =>
        {
            Err(AppError::new(
                "push_unavailable",
                BRANCH_BEHIND_REMOTE_MESSAGE,
            ))
        }
        Err(error) => Err(error),
    }
}

fn is_remote_outdated_push_error(message: &str) -> bool {
    let normalized = message.to_ascii_lowercase();
    normalized.contains("non-fast-forward")
        || normalized.contains("(fetch first)")
        || normalized.contains("remote contains work that you do not")
}

// UI 的 ahead / 历史列表采用完整 upstream..HEAD 口径，尽量贴近 GitHub Desktop；
// 分步推送和 push to commit 仍只允许 first-parent 安全路径，避免 merge 侧支被当成线性推送目标。
fn safe_unpushed_hashes(repo_path: &Path, upstream: &str) -> AppResult<HashSet<String>> {
    Ok(safe_unpushed_hashes_in_push_order(repo_path, upstream)?
        .into_iter()
        .collect())
}

fn first_parent_unpushed_hashes_in_push_order(
    repo_path: &Path,
    upstream: &str,
) -> AppResult<Vec<String>> {
    let range = format!("{upstream}..HEAD");
    let output = git_trimmed(repo_path, &["rev-list", "--first-parent", &range])?;
    let mut hashes: Vec<String> = output
        .lines()
        .map(str::trim)
        .filter(|hash| !hash.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    hashes.reverse();
    Ok(hashes)
}

fn safe_unpushed_hashes_in_push_order(repo_path: &Path, upstream: &str) -> AppResult<Vec<String>> {
    let hashes = first_parent_unpushed_hashes_in_push_order(repo_path, upstream)?;
    let mut first_safe_index = None;
    for (index, hash) in hashes.iter().enumerate() {
        if is_ancestor(repo_path, upstream, hash)? {
            first_safe_index = Some(index);
            break;
        }
    }
    let Some(first_safe_index) = first_safe_index else {
        return Ok(Vec::new());
    };

    Ok(hashes[first_safe_index..].to_vec())
}

fn ensure_safe_push_target(repo_path: &Path, hash: &str) -> AppResult<()> {
    sync_origin_tracking(repo_path)?;

    let branch_status = branch_status_for_path(repo_path)?;
    let disabled_reason = branch_status.disabled_reason.clone();
    let upstream = branch_status.upstream.as_deref().ok_or_else(|| {
        AppError::new(
            "push_unavailable",
            disabled_reason
                .clone()
                .unwrap_or_else(|| "当前分支没有上游跟踪分支，无法执行推送".to_string()),
        )
    })?;

    if !branch_status.push_available {
        return Err(AppError::new(
            "push_unavailable",
            disabled_reason.unwrap_or_else(|| "当前分支当前不可推送".to_string()),
        ));
    }

    if branch_status.behind_count > 0 {
        return Err(AppError::new(
            "push_unavailable",
            BRANCH_BEHIND_REMOTE_MESSAGE,
        ));
    }

    ensure_remote_fast_forward_target(repo_path, &branch_status.branch, hash)?;

    let safe_targets = safe_unpushed_hashes(repo_path, upstream)?;
    if safe_targets.contains(hash) {
        Ok(())
    } else {
        Err(AppError::new(
            "unsafe_push_target",
            UNSAFE_PUSH_TARGET_MESSAGE,
        ))
    }
}

fn ensure_safe_step_push_hashes(repo_path: &Path, hashes: &[String]) -> AppResult<()> {
    let Some(target_hash) = hashes.last() else {
        return Err(AppError::new("empty_hashes", "没有可推送的 Commit"));
    };

    ensure_safe_push_target(repo_path, target_hash)?;

    // 前端传入的 hashes 必须等于“从最旧安全 commit 到目标 commit”的连续前缀；
    // 这样后端即使收到被篡改的请求，也不会跳过中间必须先推送的安全节点。
    let branch_status = branch_status_for_path(repo_path)?;
    let upstream = branch_status.upstream.as_deref().ok_or_else(|| {
        AppError::new(
            "push_unavailable",
            branch_status
                .disabled_reason
                .unwrap_or_else(|| "当前分支没有上游跟踪分支，无法执行推送".to_string()),
        )
    })?;

    let safe_hashes = safe_unpushed_hashes_in_push_order(repo_path, upstream)?;
    let target_index = safe_hashes
        .iter()
        .position(|hash| hash == target_hash)
        .ok_or_else(|| AppError::new("unsafe_push_target", UNSAFE_PUSH_TARGET_MESSAGE))?;
    let expected = &safe_hashes[..=target_index];

    if expected == hashes {
        Ok(())
    } else {
        Err(AppError::new(
            "unsafe_push_target",
            UNSAFE_PUSH_TARGET_MESSAGE,
        ))
    }
}

pub fn validate_push_target(repo_path: &str, hash: &str) -> AppResult<()> {
    let repo_path = resolve_repository_path(repo_path)?;
    ensure_safe_push_target(&repo_path, hash)
}

pub fn validate_step_push_hashes(repo_path: &str, hashes: &[String]) -> AppResult<()> {
    let repo_path = resolve_repository_path(repo_path)?;
    ensure_safe_step_push_hashes(&repo_path, hashes)
}

pub fn get_commit_file_diff(
    repo_path: &str,
    hash: &str,
    file_path: &str,
    previous_path: Option<&str>,
    ignore_whitespace: bool,
) -> AppResult<String> {
    let repo_path = resolve_repository_path(repo_path)?;
    let parents = git_trimmed(&repo_path, &["show", "-s", "--format=%P", hash])?;
    let whitespace_arg = ignore_whitespace.then_some("-w");
    let mut pathspecs = Vec::new();

    if let Some(previous_path) = previous_path
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != file_path)
    {
        pathspecs.push(previous_path);
    }

    pathspecs.push(file_path);

    if parents.trim().is_empty() {
        let mut args = vec!["show"];
        if let Some(arg) = whitespace_arg {
            // 初始提交如果继续走 git show -w，不同 Git 版本下空白过滤语义不够稳定；
            // 这里改成 empty-tree -> commit 的 diff，和普通提交保持一致。
            args = vec!["diff"];
            args.push(arg);
            args.extend(["--find-renames", EMPTY_TREE_HASH, hash, "--"]);
            args.extend(pathspecs.iter().copied());
            return git_text(&repo_path, &args);
        }
        args.extend(["--find-renames", "--format=", hash, "--"]);
        args.extend(pathspecs.iter().copied());
        return git_text(&repo_path, &args);
    }

    let parent_ref = format!("{hash}^");
    let mut args = vec!["diff"];
    if let Some(arg) = whitespace_arg {
        args.push(arg);
    }
    args.extend(["--find-renames", parent_ref.as_str(), hash, "--"]);
    args.extend(pathspecs.iter().copied());
    git_text(&repo_path, &args)
}

pub fn push_current_branch(repo_path: &str) -> AppResult<()> {
    let repo_path = resolve_repository_path(repo_path)?;
    let (branch, detached) = current_branch_name(&repo_path)?;

    if detached {
        return Err(AppError::new(
            "detached_head",
            "当前仓库处于 detached HEAD 状态，无法推送当前分支",
        ));
    }

    sync_origin_tracking(&repo_path)?;

    let head = git_trimmed(&repo_path, &["rev-parse", "HEAD"])?;
    ensure_remote_fast_forward_target(&repo_path, &branch, &head)?;

    let refspec = format!("HEAD:refs/heads/{branch}");
    git_run(&repo_path, &["push", REMOTE_NAME, &refspec])
}

pub fn push_current_branch_checked(repo_path: &str, branch: &str) -> AppResult<()> {
    let repo_path = resolve_repository_path(repo_path)?;
    let branch = current_branch_matching(&repo_path, branch)?;

    sync_origin_tracking(&repo_path)?;

    let head = git_trimmed(&repo_path, &["rev-parse", "HEAD"])?;
    ensure_remote_fast_forward_target(&repo_path, &branch, &head)?;

    let refspec = format!("HEAD:refs/heads/{branch}");
    git_run(&repo_path, &["push", REMOTE_NAME, &refspec])
}

pub fn push_to_commit(repo_path: &str, branch: &str, hash: &str) -> AppResult<()> {
    let repo_path = resolve_repository_path(repo_path)?;
    let branch = current_branch_matching(&repo_path, branch)?;
    ensure_safe_push_target(&repo_path, hash)?;
    let refspec = format!("{hash}:refs/heads/{branch}");
    git_run(&repo_path, &["push", REMOTE_NAME, &refspec])
}

#[cfg(test)]
mod tests {
    use super::{
        branch_status_for_path, checkout_branch, get_commit_file_diff, get_commit_history,
        get_commit_meta, list_local_branches, push_current_branch, push_to_commit,
        refresh_remote_tracking, resolve_repository_path, validate_current_branch,
        validate_step_push_hashes, BRANCH_BEHIND_REMOTE_MESSAGE, BRANCH_MISMATCH_MESSAGE,
        UNSAFE_PUSH_TARGET_MESSAGE,
    };
    use crate::error::AppError;
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

    fn run_git(path: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .output()
            .expect("run git command");

        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }

        panic!("{}", String::from_utf8_lossy(&output.stderr));
    }

    fn init_repo() -> TestDir {
        let repo = TestDir::new("repo");
        run_git(&repo.path, &["init"]);
        run_git(&repo.path, &["config", "user.name", "TickGit Tests"]);
        run_git(
            &repo.path,
            &["config", "user.email", "tickgit-tests@example.com"],
        );
        repo
    }

    fn init_bare_repo() -> TestDir {
        let repo = TestDir::new("bare-repo");
        run_git(&repo.path, &["init", "--bare"]);
        repo
    }

    fn write_file(path: &Path, relative_path: &str, content: &str) {
        let target = path.join(relative_path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).expect("create parent directory");
        }
        fs::write(target, content).expect("write file");
    }

    fn commit_file(path: &Path, relative_path: &str, content: &str, message: &str) -> String {
        write_file(path, relative_path, content);
        run_git(path, &["add", relative_path]);
        run_git(path, &["commit", "--no-gpg-sign", "-m", message]);
        run_git(path, &["rev-parse", "HEAD"])
    }

    fn current_test_branch(path: &Path) -> String {
        run_git(path, &["branch", "--show-current"])
    }

    fn clone_repo(source: &Path, prefix: &str) -> TestDir {
        let repo = TestDir::new(prefix);
        run_git(
            source.parent().expect("clone source parent"),
            &[
                "clone",
                source.to_str().expect("clone source path"),
                repo.path.to_str().expect("clone target path"),
            ],
        );
        run_git(&repo.path, &["config", "user.name", "TickGit Tests"]);
        run_git(
            &repo.path,
            &["config", "user.email", "tickgit-tests@example.com"],
        );
        repo
    }

    fn assert_app_error(error: AppError, code: &str, message: &str) {
        assert_eq!(error.code, code);
        assert_eq!(error.message, message);
    }

    #[test]
    fn resolves_valid_repository_path() {
        let repo = init_repo();
        let resolved = resolve_repository_path(repo.path.to_string_lossy().as_ref()).unwrap();
        assert_eq!(resolved, repo.path.canonicalize().unwrap());
    }

    #[test]
    fn rejects_missing_repository_path() {
        let error = resolve_repository_path("/tmp/tickgit-missing-repo-path").unwrap_err();
        assert_app_error(error, "invalid_repository", "仓库路径不存在");
    }

    #[test]
    fn rejects_non_git_directory() {
        let dir = TestDir::new("plain-dir");
        let error = resolve_repository_path(dir.path.to_string_lossy().as_ref()).unwrap_err();
        assert_app_error(error, "invalid_repository", "当前目录不是 Git 仓库");
    }

    #[test]
    fn disables_push_when_upstream_is_not_origin() {
        let repo = init_repo();
        let origin = init_bare_repo();
        let backup = init_bare_repo();
        commit_file(&repo.path, "file.txt", "hello\n", "initial");
        let branch = current_test_branch(&repo.path);

        run_git(
            &repo.path,
            &["remote", "add", "origin", origin.path.to_str().unwrap()],
        );
        run_git(
            &repo.path,
            &["remote", "add", "backup", backup.path.to_str().unwrap()],
        );
        let backup_refspec = format!("HEAD:refs/heads/{branch}");
        run_git(&repo.path, &["push", "-u", "backup", &backup_refspec]);

        let status = branch_status_for_path(&repo.path).unwrap();
        let expected_upstream = format!("backup/{branch}");

        assert_eq!(status.upstream.as_deref(), Some(expected_upstream.as_str()));
        assert!(!status.push_available);
        assert_eq!(
            status.disabled_reason.as_deref(),
            Some("当前分支的上游不是 origin 远端，已禁用推送动作")
        );
    }

    #[test]
    fn pushes_current_branch_to_origin_even_when_upstream_points_elsewhere() {
        let repo = init_repo();
        let origin = init_bare_repo();
        let backup = init_bare_repo();
        commit_file(&repo.path, "file.txt", "hello\n", "initial");
        let branch = current_test_branch(&repo.path);

        run_git(
            &repo.path,
            &["remote", "add", "origin", origin.path.to_str().unwrap()],
        );
        run_git(
            &repo.path,
            &["remote", "add", "backup", backup.path.to_str().unwrap()],
        );
        let backup_refspec = format!("HEAD:refs/heads/{branch}");
        run_git(&repo.path, &["push", "-u", "backup", &backup_refspec]);
        let backup_initial_head = run_git(
            &backup.path,
            &["rev-parse", &format!("refs/heads/{branch}")],
        );

        let second_hash = commit_file(&repo.path, "file.txt", "hello\nworld\n", "second");

        push_current_branch(repo.path.to_string_lossy().as_ref()).unwrap();

        let origin_head = run_git(
            &origin.path,
            &["rev-parse", &format!("refs/heads/{branch}")],
        );
        let backup_head = run_git(
            &backup.path,
            &["rev-parse", &format!("refs/heads/{branch}")],
        );

        assert_eq!(origin_head, second_hash);
        assert_eq!(backup_head, backup_initial_head);
    }

    #[test]
    fn separates_total_history_from_safe_step_push_targets_after_merge() {
        let repo = init_repo();
        let origin = init_bare_repo();
        let base_hash = commit_file(&repo.path, "base.txt", "base\n", "base");
        let branch = current_test_branch(&repo.path);
        let refspec = format!("HEAD:refs/heads/{branch}");

        run_git(
            &repo.path,
            &["remote", "add", "origin", origin.path.to_str().unwrap()],
        );
        run_git(&repo.path, &["push", "-u", "origin", &refspec]);

        run_git(&repo.path, &["checkout", "-b", "feature", &base_hash]);
        let feature_hash = commit_file(&repo.path, "feature.txt", "feature\n", "feature");

        run_git(&repo.path, &["checkout", &branch]);
        let main_hash = commit_file(&repo.path, "main.txt", "main\n", "main");
        run_git(
            &repo.path,
            &["merge", "--no-ff", "feature", "-m", "merge feature"],
        );
        let merge_hash = run_git(&repo.path, &["rev-parse", "HEAD"]);

        let status = branch_status_for_path(&repo.path).unwrap();
        assert_eq!(status.ahead_count, 3);
        assert_eq!(status.safe_ahead_count, 2);
        assert_eq!(status.behind_count, 0);

        let history = get_commit_history(repo.path.to_string_lossy().as_ref(), 0, 10).unwrap();
        let hashes: Vec<&str> = history
            .items
            .iter()
            .map(|item| item.hash.as_str())
            .collect();

        assert_eq!(history.unpushed_count, 3);
        assert_eq!(history.safe_unpushed_count, 2);
        assert!(hashes.contains(&merge_hash.as_str()));
        assert!(hashes.contains(&main_hash.as_str()));
        assert!(hashes.contains(&feature_hash.as_str()));
        assert!(hashes.contains(&base_hash.as_str()));

        let merge_item = history
            .items
            .iter()
            .find(|item| item.hash == merge_hash)
            .expect("merge commit in history");
        let main_item = history
            .items
            .iter()
            .find(|item| item.hash == main_hash)
            .expect("main commit in history");
        let feature_item = history
            .items
            .iter()
            .find(|item| item.hash == feature_hash)
            .expect("feature commit in history");
        let base_item = history
            .items
            .iter()
            .find(|item| item.hash == base_hash)
            .expect("base commit in history");

        assert!(!merge_item.is_pushed);
        assert!(merge_item.is_safe_push_target);
        assert_eq!(merge_item.push_blocked_reason, None);
        assert!(!main_item.is_pushed);
        assert!(main_item.is_safe_push_target);
        assert_eq!(main_item.push_blocked_reason, None);
        assert!(!feature_item.is_pushed);
        assert!(!feature_item.is_safe_push_target);
        assert_eq!(
            feature_item.push_blocked_reason.as_deref(),
            Some(UNSAFE_PUSH_TARGET_MESSAGE)
        );
        assert!(base_item.is_pushed);
    }

    #[test]
    fn rejects_push_to_commit_for_unsafe_merge_side_branch_commit() {
        let repo = init_repo();
        let origin = init_bare_repo();
        let base_hash = commit_file(&repo.path, "base.txt", "base\n", "base");
        let branch = current_test_branch(&repo.path);
        let refspec = format!("HEAD:refs/heads/{branch}");

        run_git(
            &repo.path,
            &["remote", "add", "origin", origin.path.to_str().unwrap()],
        );
        run_git(&repo.path, &["push", "-u", "origin", &refspec]);

        run_git(&repo.path, &["checkout", "-b", "feature", &base_hash]);
        let feature_hash = commit_file(&repo.path, "feature.txt", "feature\n", "feature");

        run_git(&repo.path, &["checkout", &branch]);
        commit_file(&repo.path, "main.txt", "main\n", "main");
        run_git(
            &repo.path,
            &["merge", "--no-ff", "feature", "-m", "merge feature"],
        );

        let error = push_to_commit(repo.path.to_string_lossy().as_ref(), &branch, &feature_hash)
            .unwrap_err();
        assert_eq!(error.code, "unsafe_push_target");
        assert_eq!(error.message, UNSAFE_PUSH_TARGET_MESSAGE);
    }

    #[test]
    fn validates_requested_branch_must_match_current_branch() {
        let repo = init_repo();
        commit_file(&repo.path, "base.txt", "base\n", "base");
        let branch = current_test_branch(&repo.path);

        let current =
            validate_current_branch(repo.path.to_string_lossy().as_ref(), &branch).unwrap();
        assert_eq!(current, branch);

        let mismatch = validate_current_branch(repo.path.to_string_lossy().as_ref(), "not-current")
            .unwrap_err();
        assert_app_error(mismatch, "branch_mismatch", BRANCH_MISMATCH_MESSAGE);

        let empty =
            validate_current_branch(repo.path.to_string_lossy().as_ref(), "   ").unwrap_err();
        assert_app_error(empty, "invalid_branch", "目标分支不能为空");
    }

    #[test]
    fn rejects_push_to_commit_when_requested_branch_differs_from_current() {
        let repo = init_repo();
        let origin = init_bare_repo();
        let base_hash = commit_file(&repo.path, "base.txt", "base\n", "base");
        let branch = current_test_branch(&repo.path);
        let refspec = format!("HEAD:refs/heads/{branch}");

        run_git(
            &repo.path,
            &["remote", "add", "origin", origin.path.to_str().unwrap()],
        );
        run_git(&repo.path, &["push", "-u", "origin", &refspec]);

        let safe_hash = commit_file(&repo.path, "safe.txt", "safe\n", "safe");
        let error = push_to_commit(
            repo.path.to_string_lossy().as_ref(),
            "not-current",
            &safe_hash,
        )
        .unwrap_err();

        assert_app_error(error, "branch_mismatch", BRANCH_MISMATCH_MESSAGE);

        let origin_head = run_git(
            &origin.path,
            &["rev-parse", &format!("refs/heads/{branch}")],
        );
        assert_eq!(origin_head, base_hash);
    }

    #[test]
    fn allows_push_to_commit_for_safe_first_parent_commit() {
        let repo = init_repo();
        let origin = init_bare_repo();
        commit_file(&repo.path, "base.txt", "base\n", "base");
        let branch = current_test_branch(&repo.path);
        let refspec = format!("HEAD:refs/heads/{branch}");

        run_git(
            &repo.path,
            &["remote", "add", "origin", origin.path.to_str().unwrap()],
        );
        run_git(&repo.path, &["push", "-u", "origin", &refspec]);

        let safe_hash = commit_file(&repo.path, "safe.txt", "safe\n", "safe");

        push_to_commit(repo.path.to_string_lossy().as_ref(), &branch, &safe_hash).unwrap();

        let origin_head = run_git(
            &origin.path,
            &["rev-parse", &format!("refs/heads/{branch}")],
        );
        assert_eq!(origin_head, safe_hash);
    }

    #[test]
    fn accepts_contiguous_step_push_hashes() {
        let repo = init_repo();
        let origin = init_bare_repo();
        commit_file(&repo.path, "base.txt", "base\n", "base");
        let branch = current_test_branch(&repo.path);
        let refspec = format!("HEAD:refs/heads/{branch}");

        run_git(
            &repo.path,
            &["remote", "add", "origin", origin.path.to_str().unwrap()],
        );
        run_git(&repo.path, &["push", "-u", "origin", &refspec]);

        let first_hash = commit_file(&repo.path, "first.txt", "first\n", "first");
        let second_hash = commit_file(&repo.path, "second.txt", "second\n", "second");
        commit_file(&repo.path, "third.txt", "third\n", "third");

        validate_step_push_hashes(
            repo.path.to_string_lossy().as_ref(),
            &[first_hash, second_hash],
        )
        .unwrap();
    }

    #[test]
    fn rejects_non_contiguous_step_push_hashes() {
        let repo = init_repo();
        let origin = init_bare_repo();
        commit_file(&repo.path, "base.txt", "base\n", "base");
        let branch = current_test_branch(&repo.path);
        let refspec = format!("HEAD:refs/heads/{branch}");

        run_git(
            &repo.path,
            &["remote", "add", "origin", origin.path.to_str().unwrap()],
        );
        run_git(&repo.path, &["push", "-u", "origin", &refspec]);

        let first_hash = commit_file(&repo.path, "first.txt", "first\n", "first");
        commit_file(&repo.path, "second.txt", "second\n", "second");
        let third_hash = commit_file(&repo.path, "third.txt", "third\n", "third");

        let error = validate_step_push_hashes(
            repo.path.to_string_lossy().as_ref(),
            &[first_hash, third_hash],
        )
        .unwrap_err();

        assert_eq!(error.code, "unsafe_push_target");
        assert_eq!(error.message, UNSAFE_PUSH_TARGET_MESSAGE);
    }

    #[test]
    fn rejects_push_to_commit_when_remote_advanced_without_fetch() {
        let repo = init_repo();
        let origin = init_bare_repo();
        commit_file(&repo.path, "base.txt", "base\n", "base");
        let branch = current_test_branch(&repo.path);
        let refspec = format!("HEAD:refs/heads/{branch}");

        run_git(
            &repo.path,
            &["remote", "add", "origin", origin.path.to_str().unwrap()],
        );
        run_git(&repo.path, &["push", "-u", "origin", &refspec]);

        let peer = clone_repo(&origin.path, "peer");
        commit_file(&peer.path, "remote.txt", "remote\n", "remote");
        run_git(&peer.path, &["push", "origin", &format!("HEAD:{branch}")]);

        let local_hash = commit_file(&repo.path, "local.txt", "local\n", "local");

        let error =
            push_to_commit(repo.path.to_string_lossy().as_ref(), &branch, &local_hash).unwrap_err();

        assert_eq!(error.code, "push_unavailable");
        assert_eq!(error.message, BRANCH_BEHIND_REMOTE_MESSAGE);
    }

    #[test]
    fn rejects_step_push_validation_when_remote_advanced_without_fetch() {
        let repo = init_repo();
        let origin = init_bare_repo();
        commit_file(&repo.path, "base.txt", "base\n", "base");
        let branch = current_test_branch(&repo.path);
        let refspec = format!("HEAD:refs/heads/{branch}");

        run_git(
            &repo.path,
            &["remote", "add", "origin", origin.path.to_str().unwrap()],
        );
        run_git(&repo.path, &["push", "-u", "origin", &refspec]);

        let peer = clone_repo(&origin.path, "peer");
        commit_file(&peer.path, "remote.txt", "remote\n", "remote");
        run_git(&peer.path, &["push", "origin", &format!("HEAD:{branch}")]);

        let local_hash = commit_file(&repo.path, "local.txt", "local\n", "local");

        let error = validate_step_push_hashes(repo.path.to_string_lossy().as_ref(), &[local_hash])
            .unwrap_err();

        assert_eq!(error.code, "push_unavailable");
        assert_eq!(error.message, BRANCH_BEHIND_REMOTE_MESSAGE);
    }

    #[test]
    fn refreshes_remote_tracking_and_marks_branch_behind_remote() {
        let repo = init_repo();
        let origin = init_bare_repo();
        commit_file(&repo.path, "base.txt", "base\n", "base");
        let branch = current_test_branch(&repo.path);
        let refspec = format!("HEAD:refs/heads/{branch}");

        run_git(
            &repo.path,
            &["remote", "add", "origin", origin.path.to_str().unwrap()],
        );
        run_git(&repo.path, &["push", "-u", "origin", &refspec]);

        let peer = clone_repo(&origin.path, "peer");
        commit_file(&peer.path, "remote.txt", "remote\n", "remote");
        run_git(&peer.path, &["push", "origin", &format!("HEAD:{branch}")]);

        commit_file(&repo.path, "local.txt", "local\n", "local");

        refresh_remote_tracking(repo.path.to_string_lossy().as_ref()).unwrap();

        let status = branch_status_for_path(&repo.path).unwrap();
        assert_eq!(status.behind_count, 1);
        assert!(!status.push_available);
        assert_eq!(
            status.disabled_reason.as_deref(),
            Some(BRANCH_BEHIND_REMOTE_MESSAGE)
        );
    }

    #[test]
    fn keeps_unpushed_commits_visible_when_branch_is_behind_remote() {
        let repo = init_repo();
        let origin = init_bare_repo();
        commit_file(&repo.path, "base.txt", "base\n", "base");
        let branch = current_test_branch(&repo.path);
        let refspec = format!("HEAD:refs/heads/{branch}");

        run_git(
            &repo.path,
            &["remote", "add", "origin", origin.path.to_str().unwrap()],
        );
        run_git(&repo.path, &["push", "-u", "origin", &refspec]);

        let peer = clone_repo(&origin.path, "peer");
        commit_file(&peer.path, "remote.txt", "remote\n", "remote");
        run_git(&peer.path, &["push", "origin", &format!("HEAD:{branch}")]);

        let local_hash = commit_file(&repo.path, "local.txt", "local\n", "local");

        refresh_remote_tracking(repo.path.to_string_lossy().as_ref()).unwrap();

        let history = get_commit_history(repo.path.to_string_lossy().as_ref(), 0, 10).unwrap();
        let status = branch_status_for_path(&repo.path).unwrap();
        let local_item = history
            .items
            .iter()
            .find(|item| item.hash == local_hash)
            .expect("local commit in history");

        assert_eq!(status.behind_count, 1);
        assert!(!status.push_available);
        assert_eq!(status.safe_ahead_count, 0);
        assert!(!local_item.is_pushed);
        assert!(!local_item.is_safe_push_target);
        assert_eq!(
            local_item.push_blocked_reason.as_deref(),
            Some(BRANCH_BEHIND_REMOTE_MESSAGE)
        );
        assert_eq!(history.unpushed_count, 1);
        assert_eq!(history.safe_unpushed_count, 0);
    }

    #[test]
    fn limits_safe_step_push_targets_to_fast_forwardable_suffix_when_remote_is_merged_side_parent()
    {
        let repo = init_repo();
        let origin = init_bare_repo();
        commit_file(&repo.path, "base.txt", "base\n", "base");
        let branch = current_test_branch(&repo.path);
        let refspec = format!("HEAD:refs/heads/{branch}");

        run_git(
            &repo.path,
            &["remote", "add", "origin", origin.path.to_str().unwrap()],
        );
        run_git(&repo.path, &["push", "-u", "origin", &refspec]);

        let peer = clone_repo(&origin.path, "peer");
        commit_file(&peer.path, "remote.txt", "remote\n", "remote");
        run_git(&peer.path, &["push", "origin", &format!("HEAD:{branch}")]);

        let local_main_hash = commit_file(&repo.path, "main.txt", "main\n", "main");
        refresh_remote_tracking(repo.path.to_string_lossy().as_ref()).unwrap();
        run_git(
            &repo.path,
            &[
                "merge",
                "--no-ff",
                &format!("origin/{branch}"),
                "-m",
                "merge remote",
            ],
        );
        let merge_hash = run_git(&repo.path, &["rev-parse", "HEAD"]);

        let status = branch_status_for_path(&repo.path).unwrap();
        assert_eq!(status.behind_count, 0);
        assert_eq!(status.ahead_count, 2);
        assert_eq!(status.safe_ahead_count, 1);

        let history = get_commit_history(repo.path.to_string_lossy().as_ref(), 0, 10).unwrap();
        let merge_item = history
            .items
            .iter()
            .find(|item| item.hash == merge_hash)
            .expect("merge commit in history");
        let main_item = history
            .items
            .iter()
            .find(|item| item.hash == local_main_hash)
            .expect("main commit in history");

        assert!(merge_item.is_safe_push_target);
        assert!(!main_item.is_safe_push_target);
        assert_eq!(history.safe_unpushed_count, 1);

        validate_step_push_hashes(
            repo.path.to_string_lossy().as_ref(),
            std::slice::from_ref(&merge_hash),
        )
        .unwrap();

        let error = validate_step_push_hashes(
            repo.path.to_string_lossy().as_ref(),
            &[local_main_hash, merge_hash],
        )
        .unwrap_err();
        assert_eq!(error.code, "unsafe_push_target");
        assert_eq!(error.message, UNSAFE_PUSH_TARGET_MESSAGE);
    }

    #[test]
    fn gets_diff_for_initial_commit() {
        let repo = init_repo();
        let initial_hash = commit_file(&repo.path, "file.txt", "hello\n", "initial");

        let diff = get_commit_file_diff(
            repo.path.to_string_lossy().as_ref(),
            &initial_hash,
            "file.txt",
            None,
            false,
        )
        .unwrap();

        assert!(diff.contains("diff --git"));
        assert!(diff.contains("+hello"));
    }

    #[test]
    fn gets_diff_for_non_initial_commit() {
        let repo = init_repo();
        commit_file(&repo.path, "file.txt", "hello\n", "initial");
        let second_hash = commit_file(&repo.path, "file.txt", "hello\nworld\n", "second");

        let diff = get_commit_file_diff(
            repo.path.to_string_lossy().as_ref(),
            &second_hash,
            "file.txt",
            None,
            false,
        )
        .unwrap();

        assert!(diff.contains("@@"));
        assert!(diff.contains("+world"));
    }

    #[test]
    fn hides_whitespace_only_diff_for_non_initial_commit() {
        let repo = init_repo();
        commit_file(&repo.path, "file.txt", "hello\n", "initial");
        let second_hash = commit_file(&repo.path, "file.txt", "hello \n", "whitespace");

        let normal_diff = get_commit_file_diff(
            repo.path.to_string_lossy().as_ref(),
            &second_hash,
            "file.txt",
            None,
            false,
        )
        .unwrap();
        let hidden_diff = get_commit_file_diff(
            repo.path.to_string_lossy().as_ref(),
            &second_hash,
            "file.txt",
            None,
            true,
        )
        .unwrap();

        assert!(normal_diff.contains("@@"));
        assert!(hidden_diff.trim().is_empty());
    }

    #[test]
    fn gets_diff_for_initial_commit_when_hiding_whitespace() {
        let repo = init_repo();
        let initial_hash = commit_file(&repo.path, "file.txt", "   \n", "initial");

        let hidden_diff = get_commit_file_diff(
            repo.path.to_string_lossy().as_ref(),
            &initial_hash,
            "file.txt",
            None,
            true,
        )
        .unwrap();

        assert!(hidden_diff.contains("diff --git"));
        assert!(hidden_diff.contains("file.txt"));
    }

    #[test]
    fn gets_rename_diff_when_previous_path_is_available() {
        let repo = init_repo();
        commit_file(&repo.path, "old.txt", "hello\n", "initial");
        run_git(&repo.path, &["mv", "old.txt", "new.txt"]);
        run_git(&repo.path, &["commit", "--no-gpg-sign", "-m", "rename"]);
        let rename_hash = run_git(&repo.path, &["rev-parse", "HEAD"]);

        let diff = get_commit_file_diff(
            repo.path.to_string_lossy().as_ref(),
            &rename_hash,
            "new.txt",
            Some("old.txt"),
            false,
        )
        .unwrap();

        assert!(diff.contains("rename from old.txt"));
        assert!(diff.contains("rename to new.txt"));
    }

    #[test]
    fn gets_commit_meta() {
        let repo = init_repo();
        write_file(&repo.path, "file.txt", "before\n");
        run_git(&repo.path, &["add", "file.txt"]);
        run_git(
            &repo.path,
            &[
                "commit",
                "--no-gpg-sign",
                "-m",
                "summary",
                "-m",
                "details line",
            ],
        );
        write_file(&repo.path, "file.txt", "before\nafter\n");
        run_git(&repo.path, &["add", "file.txt"]);
        run_git(
            &repo.path,
            &[
                "commit",
                "--no-gpg-sign",
                "-m",
                "follow up",
                "-m",
                "more context",
            ],
        );

        let hash = run_git(&repo.path, &["rev-parse", "HEAD"]);
        let meta = get_commit_meta(repo.path.to_string_lossy().as_ref(), &hash).unwrap();

        assert_eq!(meta.body, "more context");
        assert_eq!(meta.additions, 1);
        assert_eq!(meta.deletions, 0);
    }

    #[test]
    fn lists_local_branches() {
        let repo = init_repo();
        commit_file(&repo.path, "file.txt", "hello\n", "initial");
        let current_branch = current_test_branch(&repo.path);
        run_git(&repo.path, &["branch", "feature"]);

        let branches = list_local_branches(repo.path.to_string_lossy().as_ref()).unwrap();

        assert_eq!(branches, vec!["feature".to_string(), current_branch]);
    }

    #[test]
    fn checks_out_selected_local_branch() {
        let repo = init_repo();
        commit_file(&repo.path, "file.txt", "hello\n", "initial");
        run_git(&repo.path, &["branch", "feature"]);

        checkout_branch(repo.path.to_string_lossy().as_ref(), "feature").unwrap();

        let status = branch_status_for_path(&repo.path).unwrap();
        assert_eq!(status.branch, "feature");
    }
}
