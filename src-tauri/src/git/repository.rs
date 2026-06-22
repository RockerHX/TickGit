use std::path::{Path, PathBuf};

use crate::{
    error::{AppError, AppResult},
    models::{BranchStatus, RepositoryRevision},
};

use super::{
    command::{git_run, git_trimmed},
    parse::{parse_ahead_behind, parse_count},
    push::safe_unpushed_hashes,
    BRANCH_BEHIND_REMOTE_MESSAGE, BRANCH_MISMATCH_MESSAGE, REMOTE_NAME,
};

pub(super) fn current_branch_name(repo_path: &Path) -> AppResult<(String, bool)> {
    let branch = git_trimmed(repo_path, &["branch", "--show-current"])?;

    if branch.is_empty() {
        Ok(("HEAD".to_string(), true))
    } else {
        Ok((branch, false))
    }
}

pub(super) fn current_branch_matching(repo_path: &Path, branch: &str) -> AppResult<String> {
    let requested_branch = branch.trim();

    if requested_branch.is_empty() {
        return Err(AppError::new("invalid_branch", "目标分支不能为空"));
    }

    let (current_branch, detached) = current_branch_name(repo_path)?;

    if detached {
        return Err(AppError::new(
            "detached_head",
            "当前仓库处于 detached HEAD 状态，无法推送当前分支",
        ));
    }

    if requested_branch != current_branch {
        return Err(AppError::new("branch_mismatch", BRANCH_MISMATCH_MESSAGE));
    }

    Ok(current_branch)
}

pub(super) fn remote_origin_exists(repo_path: &Path) -> bool {
    git_trimmed(repo_path, &["remote", "get-url", REMOTE_NAME]).is_ok()
}

pub(super) fn upstream_name(repo_path: &Path) -> Option<String> {
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

pub(super) fn upstream_is_origin(upstream: &str) -> bool {
    upstream.starts_with("origin/")
}

pub(super) fn sync_origin_tracking(repo_path: &Path) -> AppResult<()> {
    let (_, detached) = current_branch_name(repo_path)?;
    if detached || !remote_origin_exists(repo_path) {
        return Ok(());
    }

    let Some(upstream) = upstream_name(repo_path) else {
        return Ok(());
    };

    if !upstream_is_origin(&upstream) {
        return Ok(());
    }

    git_run(repo_path, &["fetch", "--prune", REMOTE_NAME])
}

pub(super) fn ahead_behind(repo_path: &Path, upstream: &str) -> AppResult<(usize, usize)> {
    let range = format!("{upstream}...HEAD");
    let counts = git_trimmed(repo_path, &["rev-list", "--left-right", "--count", &range])?;
    Ok(parse_ahead_behind(&counts))
}

pub(super) fn total_ahead_count(repo_path: &Path, upstream: &str) -> AppResult<usize> {
    let range = format!("{upstream}..HEAD");
    let output = git_trimmed(repo_path, &["rev-list", "--count", &range])?;
    Ok(parse_count(&output))
}

pub(super) fn branch_status_for_path(repo_path: &Path) -> AppResult<BranchStatus> {
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
            let ahead_count = total_ahead_count(repo_path, upstream)?;
            (ahead_count, behind_count)
        }
        None => (0, 0),
    };
    let safe_ahead_count = match upstream.as_deref() {
        Some(upstream) => safe_unpushed_hashes(repo_path, upstream)?.len(),
        None => 0,
    };

    let upstream_uses_origin = upstream.as_deref().is_some_and(upstream_is_origin);
    let push_available = !detached && has_origin && upstream_uses_origin && behind_count == 0;
    let safe_ahead_count = if behind_count > 0 {
        0
    } else {
        safe_ahead_count
    };
    let (disabled_reason, disabled_reason_code) = if detached {
        (
            Some("当前仓库处于 detached HEAD 状态，已禁用推送动作".to_string()),
            Some("detached_head".to_string()),
        )
    } else if !has_origin {
        (
            Some("当前仓库未配置 origin 远端，已禁用推送动作".to_string()),
            Some("missing_origin".to_string()),
        )
    } else if upstream.is_none() {
        (
            Some("当前分支没有上游跟踪分支，已禁用推送动作".to_string()),
            Some("missing_upstream".to_string()),
        )
    } else if !upstream_uses_origin {
        (
            Some("当前分支的上游不是 origin 远端，已禁用推送动作".to_string()),
            Some("non_origin_upstream".to_string()),
        )
    } else if behind_count > 0 {
        (
            Some(BRANCH_BEHIND_REMOTE_MESSAGE.to_string()),
            Some("behind_remote".to_string()),
        )
    } else {
        (None, None)
    };

    Ok(BranchStatus {
        branch,
        upstream,
        ahead_count,
        safe_ahead_count,
        behind_count,
        detached,
        push_available,
        disabled_reason,
        disabled_reason_code,
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

pub fn validate_current_branch(repo_path: &str, branch: &str) -> AppResult<String> {
    let repo_path = resolve_repository_path(repo_path)?;
    current_branch_matching(&repo_path, branch)
}

pub fn refresh_remote_tracking(repo_path: &str) -> AppResult<()> {
    let repo_path = resolve_repository_path(repo_path)?;
    sync_origin_tracking(&repo_path)
}

pub fn get_branch_status(repo_path: &str) -> AppResult<BranchStatus> {
    let repo_path = resolve_repository_path(repo_path)?;
    branch_status_for_path(&repo_path)
}

pub fn list_local_branches(repo_path: &str) -> AppResult<Vec<String>> {
    let repo_path = resolve_repository_path(repo_path)?;
    list_local_branches_for_path(&repo_path)
}

pub(super) fn list_local_branches_for_path(repo_path: &Path) -> AppResult<Vec<String>> {
    let output = git_trimmed(
        repo_path,
        &[
            "for-each-ref",
            "--sort=refname",
            "--format=%(refname:short)",
            "refs/heads",
        ],
    )?;

    Ok(output
        .lines()
        .map(str::trim)
        .filter(|branch| !branch.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

pub fn checkout_branch(repo_path: &str, branch: &str) -> AppResult<()> {
    let repo_path = resolve_repository_path(repo_path)?;
    let branch = branch.trim();

    if branch.is_empty() {
        return Err(AppError::new("invalid_branch", "目标分支不能为空"));
    }

    git_run(&repo_path, &["checkout", branch])
}

pub fn get_repository_revision(repo_path: &str) -> AppResult<RepositoryRevision> {
    let repo_path = resolve_repository_path(repo_path)?;
    let head = git_trimmed(&repo_path, &["rev-parse", "HEAD"])?;
    let (branch, detached) = current_branch_name(&repo_path)?;
    let upstream = if detached {
        None
    } else {
        git_trimmed(&repo_path, &["rev-parse", "@{upstream}"])
            .ok()
            .filter(|value| !value.is_empty())
    };

    Ok(RepositoryRevision {
        head,
        branch,
        upstream,
    })
}
