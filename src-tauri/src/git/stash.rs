use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    command::{git_run, git_trimmed},
    repository::{current_branch_matching, resolve_repository_path},
};
use crate::{
    error::{AppError, AppResult},
    models::{StashEntry, StashState},
};

pub(super) fn worktree_status(repo_path: &Path) -> AppResult<String> {
    git_trimmed(
        repo_path,
        &["status", "--porcelain", "--untracked-files=all"],
    )
}

fn has_tracked_changes(status: &str) -> bool {
    status.lines().any(|record| !record.starts_with("?? "))
}

fn find_stash_hash(repo_path: &Path, marker: &str) -> AppResult<Option<String>> {
    let stashes = git_trimmed(repo_path, &["stash", "list", "--format=%H%x1f%gs"])?;

    Ok(stashes.lines().find_map(|line| {
        let (hash, subject) = line.split_once('\u{1f}')?;
        subject.contains(marker).then(|| hash.to_string())
    }))
}

pub(super) fn stash_worktree_changes(
    repo_path: &Path,
    operation_id: &str,
    branch: &str,
) -> AppResult<Option<String>> {
    if worktree_status(repo_path)?.is_empty() {
        return Ok(None);
    }

    // GitHub Desktop only lists stashes whose subject ends with this marker.
    let marker = format!("TickGit working changes {operation_id}: !!GitHub_Desktop<{branch}>");
    let stash_result = git_run(
        repo_path,
        &["stash", "push", "--include-untracked", "--message", &marker],
    );
    let stash_hash = find_stash_hash(repo_path, &marker)?;
    let Some(stash_hash) = stash_hash else {
        return Err(AppError::new(
            "rebase_stash_failed",
            format!(
                "无法确认改动已储藏；请检查 git stash list 中的 `{marker}`。{}",
                stash_result
                    .err()
                    .map_or_else(String::new, |error| error.message)
            ),
        ));
    };

    if let Err(error) = stash_result {
        return Err(AppError::new(
            "rebase_stash_failed",
            format!(
                "储藏工作区改动时出错。stash {stash_hash} 已保留。{}",
                error.message
            ),
        ));
    }

    // Stashing a staged .gitignore can expose ignored generated files as untracked.
    let remaining = worktree_status(repo_path)?;
    if has_tracked_changes(&remaining) {
        return Err(AppError::new(
            "rebase_stash_incomplete",
            format!("仍有已跟踪文件的改动无法储藏，stash {stash_hash} 已保留。"),
        ));
    }

    Ok(Some(stash_hash))
}

pub(super) fn restore_stash(repo_path: &Path, stash_hash: &str) -> AppResult<()> {
    let stashes = git_trimmed(repo_path, &["stash", "list", "--format=%gd%x1f%H"])?;
    let selector = stashes.lines().find_map(|line| {
        let (selector, hash) = line.split_once('\u{1f}')?;
        (hash == stash_hash).then(|| selector.to_string())
    });
    let selector = selector.ok_or_else(|| {
        AppError::new(
            "rebase_stash_missing",
            format!("暂存的工作区改动仍保存在 stash {stash_hash}，但无法找到对应引用。"),
        )
    })?;

    git_run(repo_path, &["stash", "pop", "--index", &selector])
}

pub(super) fn ensure_no_git_operation_in_progress(repo_path: &Path) -> AppResult<()> {
    for marker in [
        "rebase-merge",
        "rebase-apply",
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
    ] {
        let path = git_trimmed(repo_path, &["rev-parse", "--git-path", marker])?;
        let path = Path::new(&path);
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_path.join(path)
        };
        if path.exists() {
            return Err(AppError::new(
                "rebase_operation_in_progress",
                "仓库已有 merge、rebase 或 cherry-pick 操作正在进行，请先完成或取消后重试。",
            ));
        }
    }

    Ok(())
}

fn repository_root(repo_path: &str) -> AppResult<PathBuf> {
    let path = resolve_repository_path(repo_path)?;
    Ok(PathBuf::from(git_trimmed(
        &path,
        &["rev-parse", "--show-toplevel"],
    )?))
}

fn branch_stashes(repo_path: &Path, branch: &str) -> AppResult<Vec<StashEntry>> {
    let output = git_trimmed(repo_path, &["stash", "list", "--format=%H%x1f%gs"])?;
    let suffix = format!("!!GitHub_Desktop<{branch}>");
    let legacy_prefix = format!("On {branch}: TickGit rebase recovery ");
    Ok(output
        .lines()
        .filter_map(|line| {
            let (hash, message) = line.split_once('\u{1f}')?;
            (message.ends_with(&suffix) || message.starts_with(&legacy_prefix)).then(|| {
                StashEntry {
                    hash: hash.to_string(),
                    message: message.to_string(),
                }
            })
        })
        .collect())
}

pub fn get_stash_state(repo_path: &str, branch: &str) -> AppResult<StashState> {
    let repo_path = repository_root(repo_path)?;
    let branch = current_branch_matching(&repo_path, branch)?;
    let status = worktree_status(&repo_path)?;
    Ok(StashState {
        has_changes: !status.is_empty(),
        can_restore: !has_tracked_changes(&status),
        entries: branch_stashes(&repo_path, &branch)?,
        branch,
    })
}

pub fn stash_current_worktree(repo_path: &str, branch: &str) -> AppResult<String> {
    let repo_path = repository_root(repo_path)?;
    let branch = current_branch_matching(&repo_path, branch)?;
    ensure_no_git_operation_in_progress(&repo_path)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    stash_worktree_changes(&repo_path, &format!("manual-{nonce}"), &branch)?
        .ok_or_else(|| AppError::new("stash_no_changes", "工作区没有需要储藏的改动。"))
}

pub fn restore_worktree_stash(repo_path: &str, branch: &str, hash: &str) -> AppResult<()> {
    let repo_path = repository_root(repo_path)?;
    let branch = current_branch_matching(&repo_path, branch)?;
    ensure_no_git_operation_in_progress(&repo_path)?;
    if has_tracked_changes(&worktree_status(&repo_path)?) {
        return Err(AppError::new(
            "stash_dirty_worktree",
            "工作区还有改动，请先提交或储藏，再恢复已有储藏。",
        ));
    }
    if !branch_stashes(&repo_path, &branch)?
        .iter()
        .any(|entry| entry.hash == hash)
    {
        return Err(AppError::new(
            "stash_not_found",
            "找不到当前分支对应的储藏，请刷新列表。",
        ));
    }
    restore_stash(&repo_path, hash).map_err(|error| {
        AppError::new(
            "stash_restore_failed",
            format!(
                "恢复未完成，stash {hash} 已保留，请检查工作区冲突。{}",
                error.message
            ),
        )
    })
}
