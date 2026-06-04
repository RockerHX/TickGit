use std::{collections::HashSet, path::Path, process::Command};

use crate::error::{AppError, AppResult};

use super::{
    command::{git_run, git_trimmed},
    repository::{
        branch_status_for_path, current_branch_matching, current_branch_name,
        resolve_repository_path, sync_origin_tracking,
    },
    BRANCH_BEHIND_REMOTE_MESSAGE, REMOTE_NAME, UNSAFE_PUSH_TARGET_MESSAGE,
};

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

pub(super) fn safe_unpushed_hashes(repo_path: &Path, upstream: &str) -> AppResult<HashSet<String>> {
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
