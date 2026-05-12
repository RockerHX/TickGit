use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    error::{AppError, AppResult},
    models::{BranchStatus, CommitFileChange, CommitHistoryPage, CommitListItem, CommitMeta},
};

const REMOTE_NAME: &str = "origin";
// Git 约定的空树对象。初始提交没有 parent 时，使用它与目标提交做 diff，
// 才能和普通提交一样统一走 diff 参数（例如 -w 忽略空白）逻辑。
const EMPTY_TREE_HASH: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

#[derive(Clone, Copy)]
enum OutputMode {
    // 只关心命令成功/失败，不消费 stdout 文本。
    Command,
    // 保留完整文本，适合 diff 这类需要保留换行的输出。
    Text,
    // 返回裁剪后的纯文本，适合分支名、计数、hash 列表等解析场景。
    TrimmedText,
}

fn git_output_bytes(repo_path: &Path, args: &[&str]) -> AppResult<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(args)
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat")
        .env("PAGER", "cat")
        .output()
        .map_err(|error| AppError::new("git_unavailable", error.to_string()))?;

    if output.status.success() {
        return Ok(output.stdout);
    }

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

fn git_command(repo_path: &Path, mode: OutputMode, args: &[&str]) -> AppResult<String> {
    let mut command = Command::new("git");

    if matches!(mode, OutputMode::Text | OutputMode::TrimmedText) {
        command.arg("--no-pager").arg("-c").arg("color.ui=false");
    }

    let output = command
        .arg("-C")
        .arg(repo_path)
        .args(args)
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat")
        .env("PAGER", "cat")
        .output()
        .map_err(|error| AppError::new("git_unavailable", error.to_string()))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        return Ok(match mode {
            OutputMode::TrimmedText => stdout.trim().to_string(),
            OutputMode::Command | OutputMode::Text => stdout,
        });
    }

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

fn git_text(repo_path: &Path, args: &[&str]) -> AppResult<String> {
    git_command(repo_path, OutputMode::Text, args)
}

fn git_trimmed(repo_path: &Path, args: &[&str]) -> AppResult<String> {
    git_command(repo_path, OutputMode::TrimmedText, args)
}

fn git_run(repo_path: &Path, args: &[&str]) -> AppResult<()> {
    let _ = git_command(repo_path, OutputMode::Command, args)?;
    Ok(())
}

fn parse_ahead_behind(counts: &str) -> (usize, usize) {
    let mut parts = counts.split_whitespace();
    let behind = parts
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_default();
    let ahead = parts
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_default();
    (ahead, behind)
}

fn parse_count(output: &str) -> usize {
    output.trim().parse::<usize>().unwrap_or_default()
}

fn parse_unpushed_hashes(output: &str) -> HashSet<String> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect()
}

fn parse_commit_tags(decorations: &str) -> Vec<String> {
    decorations
        .split(',')
        .filter_map(|part| part.trim().strip_prefix("tag: "))
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_commit_history(output: &str, unpushed: &HashSet<String>) -> Vec<CommitListItem> {
    output
        // git log 使用 record separator / unit separator，避免正文里的普通换行或空格干扰解析。
        .split('\u{1e}')
        .filter(|record| !record.trim().is_empty())
        .filter_map(|record| {
            let fields: Vec<&str> = record.split('\u{1f}').collect();
            if fields.len() < 8 {
                return None;
            }

            let hash = fields[0].trim().to_string();

            Some(CommitListItem {
                short_hash: fields[1].trim().to_string(),
                summary: fields[2].trim().to_string(),
                author_name: fields[3].trim().to_string(),
                author_email: fields[4].trim().to_string(),
                committed_at: fields[5].trim().to_string(),
                tags: parse_commit_tags(fields[6]),
                parents: fields[7]
                    .split_whitespace()
                    .map(ToOwned::to_owned)
                    .collect(),
                is_pushed: !unpushed.contains(&hash),
                hash,
            })
        })
        .collect()
}

fn parse_commit_files(output: &[u8]) -> Vec<CommitFileChange> {
    let mut files = Vec::new();
    let mut parts = output
        .split(|byte| *byte == b'\0')
        .filter(|part| !part.is_empty());

    while let Some(status_bytes) = parts.next() {
        let status = String::from_utf8_lossy(status_bytes).trim().to_string();

        if status.starts_with('R') || status.starts_with('C') {
            let previous_path = parts
                .next()
                .map(|value| String::from_utf8_lossy(value).to_string())
                .unwrap_or_default();
            let path = parts
                .next()
                .map(|value| String::from_utf8_lossy(value).to_string())
                .unwrap_or_default();

            files.push(CommitFileChange {
                display_path: format!("{previous_path} -> {path}"),
                previous_path: Some(previous_path),
                path,
                status,
            });
        } else {
            let path = parts
                .next()
                .map(|value| String::from_utf8_lossy(value).to_string())
                .unwrap_or_default();
            files.push(CommitFileChange {
                display_path: path.clone(),
                previous_path: None,
                path,
                status,
            });
        }
    }

    files
}

fn parse_shortstat(output: &str) -> (usize, usize) {
    let mut additions = 0;
    let mut deletions = 0;

    for segment in output.split(',') {
        let trimmed = segment.trim();

        if trimmed.contains("insertion") {
            additions = trimmed
                .split_whitespace()
                .next()
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or_default();
        }

        if trimmed.contains("deletion") {
            deletions = trimmed
                .split_whitespace()
                .next()
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or_default();
        }
    }

    (additions, deletions)
}

fn current_branch_name(repo_path: &Path) -> AppResult<(String, bool)> {
    let branch = git_trimmed(repo_path, &["branch", "--show-current"])?;

    if branch.is_empty() {
        Ok(("HEAD".to_string(), true))
    } else {
        Ok((branch, false))
    }
}

fn remote_origin_exists(repo_path: &Path) -> bool {
    git_trimmed(repo_path, &["remote", "get-url", REMOTE_NAME]).is_ok()
}

fn upstream_name(repo_path: &Path) -> Option<String> {
    git_trimmed(
        repo_path,
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
    )
    .ok()
    .filter(|value| !value.is_empty())
}

fn upstream_is_origin(upstream: &str) -> bool {
    upstream.starts_with("origin/")
}

fn ahead_behind(repo_path: &Path, upstream: &str) -> AppResult<(usize, usize)> {
    let range = format!("{upstream}...HEAD");
    let counts = git_trimmed(repo_path, &["rev-list", "--left-right", "--count", &range])?;
    Ok(parse_ahead_behind(&counts))
}

fn first_parent_ahead_count(repo_path: &Path, upstream: &str) -> AppResult<usize> {
    let range = format!("{upstream}..HEAD");
    let output = git_trimmed(repo_path, &["rev-list", "--first-parent", "--count", &range])?;
    Ok(parse_count(&output))
}

fn unpushed_hashes(repo_path: &Path, upstream: &str) -> AppResult<HashSet<String>> {
    let range = format!("{upstream}..HEAD");
    let output = git_trimmed(repo_path, &["rev-list", "--first-parent", &range])?;
    Ok(parse_unpushed_hashes(&output))
}

fn branch_status_for_path(repo_path: &Path) -> AppResult<BranchStatus> {
    let (branch, detached) = current_branch_name(repo_path)?;
    let has_origin = remote_origin_exists(repo_path);
    let upstream = if detached || !has_origin {
        None
    } else {
        upstream_name(repo_path)
    };

    let (ahead_count, behind_count) = match upstream.as_deref() {
        Some(upstream) => {
            let (_, behind_count) = ahead_behind(repo_path, upstream)?;
            let ahead_count = first_parent_ahead_count(repo_path, upstream)?;
            (ahead_count, behind_count)
        }
        None => (0, 0),
    };

    let upstream_uses_origin = upstream.as_deref().is_some_and(upstream_is_origin);
    let push_available = !detached && has_origin && upstream_uses_origin;
    let disabled_reason = if detached {
        Some("当前仓库处于 detached HEAD 状态，已禁用推送动作".to_string())
    } else if !has_origin {
        Some("当前仓库未配置 origin 远端，已禁用推送动作".to_string())
    } else if upstream.is_none() {
        Some("当前分支没有上游跟踪分支，已禁用推送动作".to_string())
    } else if !upstream_uses_origin {
        Some("当前分支的上游不是 origin 远端，已禁用推送动作".to_string())
    } else {
        None
    };

    Ok(BranchStatus {
        branch,
        upstream,
        ahead_count,
        behind_count,
        detached,
        push_available,
        disabled_reason,
    })
}

fn map_repository_check_error(error: AppError) -> AppError {
    if error.code == "git_command_failed" {
        AppError::new("invalid_repository", "当前目录不是 Git 仓库")
    } else {
        error
    }
}

pub fn resolve_repository_path(repo_path: &str) -> AppResult<PathBuf> {
    let path = Path::new(repo_path);

    if !path.exists() {
        return Err(AppError::new("invalid_repository", "仓库路径不存在"));
    }

    if !path.is_dir() {
        return Err(AppError::new("invalid_repository", "当前路径不是文件夹"));
    }

    let canonical_path = path
        .canonicalize()
        .map_err(|error| AppError::new("invalid_repository", error.to_string()))?;

    // 不依赖 `.git` 目录是否直接存在，统一交给 git 自己判断当前路径是否落在有效 work tree 内。
    let inside_work_tree = git_trimmed(&canonical_path, &["rev-parse", "--is-inside-work-tree"])
        .map_err(map_repository_check_error)?;

    if inside_work_tree != "true" {
        return Err(AppError::new("invalid_repository", "当前目录不是 Git 仓库"));
    }

    Ok(canonical_path)
}

pub fn validate_repository(repo_path: &str) -> AppResult<()> {
    let _ = resolve_repository_path(repo_path)?;
    Ok(())
}

pub fn get_branch_status(repo_path: &str) -> AppResult<BranchStatus> {
    let repo_path = resolve_repository_path(repo_path)?;
    branch_status_for_path(&repo_path)
}

pub fn get_commit_history(
    repo_path: &str,
    skip: usize,
    limit: usize,
) -> AppResult<CommitHistoryPage> {
    let repo_path = resolve_repository_path(repo_path)?;
    let branch_status = branch_status_for_path(&repo_path)?;
    let unpushed = match branch_status.upstream.as_deref() {
        Some(upstream) if branch_status.push_available => unpushed_hashes(&repo_path, upstream)?,
        _ => HashSet::new(),
    };

    let output = git_trimmed(
        &repo_path,
        &[
            "log",
            "--first-parent",
            "--skip",
            &skip.to_string(),
            "-n",
            &limit.to_string(),
            "--date=iso-strict",
            "--decorate=short",
            "--pretty=format:%H%x1f%h%x1f%s%x1f%an%x1f%ae%x1f%cI%x1f%D%x1f%P%x1e",
            "HEAD",
        ],
    )?;

    let items = parse_commit_history(&output, &unpushed);
    let item_count = items.len();

    Ok(CommitHistoryPage {
        items,
        next_skip: skip + item_count,
        has_more: item_count == limit,
        unpushed_count: unpushed.len(),
    })
}

pub fn get_commit_files(repo_path: &str, hash: &str) -> AppResult<Vec<CommitFileChange>> {
    let repo_path = resolve_repository_path(repo_path)?;
    let output = git_output_bytes(
        &repo_path,
        &["show", "--find-renames", "--name-status", "-z", "--format=", hash],
    )?;
    Ok(parse_commit_files(&output))
}

pub fn get_commit_meta(repo_path: &str, hash: &str) -> AppResult<CommitMeta> {
    let repo_path = resolve_repository_path(repo_path)?;
    let body = git_text(&repo_path, &["show", "-s", "--format=%b", hash])?
        .trim()
        .to_string();
    let shortstat = git_trimmed(&repo_path, &["show", "--shortstat", "--format=", hash])?;
    let (additions, deletions) = parse_shortstat(&shortstat);

    Ok(CommitMeta {
        body,
        additions,
        deletions,
    })
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

    let refspec = format!("HEAD:refs/heads/{branch}");
    git_run(&repo_path, &["push", REMOTE_NAME, &refspec])
}

pub fn push_to_commit(repo_path: &str, branch: &str, hash: &str) -> AppResult<()> {
    let repo_path = resolve_repository_path(repo_path)?;
    let refspec = format!("{hash}:refs/heads/{branch}");
    git_run(&repo_path, &["push", REMOTE_NAME, &refspec])
}

#[cfg(test)]
mod tests {
    use super::{
        branch_status_for_path, get_commit_file_diff, get_commit_history, get_commit_meta,
        parse_ahead_behind, parse_commit_files, parse_commit_history, parse_shortstat,
        push_current_branch, resolve_repository_path,
    };
    use crate::error::AppError;
    use std::{
        collections::HashSet,
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
    fn parses_ahead_behind_counts() {
        assert_eq!(parse_ahead_behind("3\t2"), (2, 3));
        assert_eq!(parse_ahead_behind("0 0"), (0, 0));
        assert_eq!(parse_ahead_behind("bad"), (0, 0));
    }

    #[test]
    fn parses_commit_history_records() {
        let unpushed = HashSet::from([String::from("hash-2")]);
        let output = concat!(
            "hash-1\x1fshort-1\x1fInitial commit\x1fAlice\x1falice@example.com\x1f2026-04-25T10:00:00Z\x1ftag: v1.0.0\x1f\x1e",
            "hash-2\x1fshort-2\x1fAdd file\x1fBob\x1fbob@example.com\x1f2026-04-25T11:00:00Z\x1fHEAD -> main, tag: v1.1.0, origin/main, tag: latest\x1fhash-1\x1e",
        );

        let items = parse_commit_history(output, &unpushed);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].hash, "hash-1");
        assert!(items[0].is_pushed);
        assert_eq!(items[0].tags, vec!["v1.0.0"]);
        assert_eq!(items[0].parents, Vec::<String>::new());
        assert_eq!(items[1].hash, "hash-2");
        assert!(!items[1].is_pushed);
        assert_eq!(items[1].tags, vec!["v1.1.0", "latest"]);
        assert_eq!(items[1].parents, vec!["hash-1"]);
    }

    #[test]
    fn parses_commit_file_changes() {
        let output = b"A\0added.txt\0M\0modified.txt\0D\0removed.txt\0R100\0old.txt\0new.txt\0C100\0source.txt\0copied.txt\0";

        let files = parse_commit_files(output);

        assert_eq!(files.len(), 5);
        assert_eq!(files[0].display_path, "added.txt");
        assert_eq!(files[3].previous_path.as_deref(), Some("old.txt"));
        assert_eq!(files[3].path, "new.txt");
        assert_eq!(files[4].display_path, "source.txt -> copied.txt");
    }

    #[test]
    fn parses_commit_files_with_tabs_in_paths() {
        let output = b"R100\0old\tname.txt\0new\tname.txt\0";

        let files = parse_commit_files(output);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].status, "R100");
        assert_eq!(files[0].previous_path.as_deref(), Some("old\tname.txt"));
        assert_eq!(files[0].path, "new\tname.txt");
        assert_eq!(files[0].display_path, "old\tname.txt -> new\tname.txt");
    }

    #[test]
    fn parses_shortstat_counts() {
        assert_eq!(parse_shortstat(" 1 file changed, 3 insertions(+), 2 deletions(-)"), (3, 2));
        assert_eq!(parse_shortstat(" 1 file changed, 4 insertions(+)"), (4, 0));
        assert_eq!(parse_shortstat(" 1 file changed, 7 deletions(-)"), (0, 7));
        assert_eq!(parse_shortstat(""), (0, 0));
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
    fn uses_first_parent_history_and_ahead_count_after_merge() {
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
        assert_eq!(status.ahead_count, 2);
        assert_eq!(status.behind_count, 0);

        let history = get_commit_history(repo.path.to_string_lossy().as_ref(), 0, 10).unwrap();
        let hashes: Vec<&str> = history.items.iter().map(|item| item.hash.as_str()).collect();

        assert_eq!(history.unpushed_count, 2);
        assert_eq!(hashes, vec![merge_hash.as_str(), main_hash.as_str(), base_hash.as_str()]);
        assert!(!hashes.contains(&feature_hash.as_str()));
        assert!(!history.items[0].is_pushed);
        assert!(!history.items[1].is_pushed);
        assert!(history.items[2].is_pushed);
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
            &["commit", "--no-gpg-sign", "-m", "summary", "-m", "details line"],
        );
        write_file(&repo.path, "file.txt", "before\nafter\n");
        run_git(&repo.path, &["add", "file.txt"]);
        run_git(
            &repo.path,
            &["commit", "--no-gpg-sign", "-m", "follow up", "-m", "more context"],
        );

        let hash = run_git(&repo.path, &["rev-parse", "HEAD"]);
        let meta = get_commit_meta(repo.path.to_string_lossy().as_ref(), &hash).unwrap();

        assert_eq!(meta.body, "more context");
        assert_eq!(meta.additions, 1);
        assert_eq!(meta.deletions, 0);
    }
}
