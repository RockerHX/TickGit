use std::{collections::HashSet, path::Path, process::Command};

use crate::{
    error::{AppError, AppResult},
    models::{BranchStatus, CommitFileChange, CommitHistoryPage, CommitListItem},
};

fn git_command(repo_path: &str, args: &[&str]) -> AppResult<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|error| AppError::new("git_unavailable", error.to_string()))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
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

pub fn validate_repository(repo_path: &str) -> AppResult<()> {
    let path = Path::new(repo_path);

    if !path.is_dir() {
        return Err(AppError::new("invalid_repository", "仓库路径不存在"));
    }

    if !path.join(".git").exists() {
        return Err(AppError::new("invalid_repository", "当前目录不是 Git 仓库"));
    }

    let _ = git_command(repo_path, &["--version"])?;
    Ok(())
}

fn remote_origin_exists(repo_path: &str) -> bool {
    git_command(repo_path, &["remote", "get-url", "origin"]).is_ok()
}

fn current_branch_name(repo_path: &str) -> AppResult<(String, bool)> {
    let branch = git_command(repo_path, &["branch", "--show-current"])?;

    if branch.is_empty() {
        Ok(("HEAD".to_string(), true))
    } else {
        Ok((branch, false))
    }
}

fn upstream_name(repo_path: &str) -> Option<String> {
    git_command(
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

fn ahead_behind(repo_path: &str, upstream: &str) -> AppResult<(usize, usize)> {
    let counts = git_command(
        repo_path,
        &[
            "rev-list",
            "--left-right",
            "--count",
            &format!("{upstream}...HEAD"),
        ],
    )?;
    let mut parts = counts.split_whitespace();
    let behind = parts
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_default();
    let ahead = parts
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_default();
    Ok((ahead, behind))
}

fn unpushed_hashes(repo_path: &str, upstream: &str) -> AppResult<HashSet<String>> {
    let output = git_command(repo_path, &["rev-list", &format!("{upstream}..HEAD")])?;
    Ok(output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect())
}

pub fn get_branch_status(repo_path: &str, _branch: Option<String>) -> AppResult<BranchStatus> {
    validate_repository(repo_path)?;

    let (branch, detached) = current_branch_name(repo_path)?;
    let has_origin = remote_origin_exists(repo_path);
    let upstream = if detached || !has_origin {
        None
    } else {
        upstream_name(repo_path)
    };

    let (ahead_count, behind_count) = match upstream.as_deref() {
        Some(upstream) => ahead_behind(repo_path, upstream)?,
        None => (0, 0),
    };

    let push_available = !detached && has_origin && upstream.is_some();
    let disabled_reason = if detached {
        Some("当前仓库处于 detached HEAD 状态，已禁用推送动作".to_string())
    } else if !has_origin {
        Some("当前仓库未配置 origin 远端，已禁用推送动作".to_string())
    } else if upstream.is_none() {
        Some("当前分支没有上游跟踪分支，已禁用推送动作".to_string())
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

pub fn get_commit_history(
    repo_path: &str,
    skip: usize,
    limit: usize,
) -> AppResult<CommitHistoryPage> {
    validate_repository(repo_path)?;
    let branch_status = get_branch_status(repo_path, None)?;
    let unpushed = match branch_status.upstream.as_deref() {
        Some(upstream) if branch_status.push_available => unpushed_hashes(repo_path, upstream)?,
        _ => HashSet::new(),
    };

    let output = git_command(
        repo_path,
        &[
            "log",
            "--skip",
            &skip.to_string(),
            "-n",
            &limit.to_string(),
            "--date=iso-strict",
            "--pretty=format:%H%x1f%h%x1f%s%x1f%an%x1f%ae%x1f%cI%x1f%P%x1e",
            "HEAD",
        ],
    )?;

    let items = output
        .split('\u{1e}')
        .filter(|record| !record.trim().is_empty())
        .filter_map(|record| {
            let fields: Vec<&str> = record.split('\u{1f}').collect();
            if fields.len() < 7 {
                return None;
            }

            Some(CommitListItem {
                hash: fields[0].trim().to_string(),
                short_hash: fields[1].trim().to_string(),
                summary: fields[2].trim().to_string(),
                author_name: fields[3].trim().to_string(),
                author_email: fields[4].trim().to_string(),
                committed_at: fields[5].trim().to_string(),
                parents: fields[6]
                    .split_whitespace()
                    .map(ToOwned::to_owned)
                    .collect(),
                is_pushed: !unpushed.contains(fields[0].trim()),
            })
        })
        .collect::<Vec<_>>();

    let item_count = items.len();

    Ok(CommitHistoryPage {
        items,
        next_skip: skip + item_count,
        has_more: item_count == limit,
        unpushed_count: unpushed.len(),
    })
}

pub fn get_commit_files(repo_path: &str, hash: &str) -> AppResult<Vec<CommitFileChange>> {
    validate_repository(repo_path)?;

    let output = git_command(repo_path, &["show", "--name-status", "--format=", hash])?;
    let mut files = Vec::new();

    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.split('\t');
        let status = parts.next().unwrap_or_default().trim().to_string();

        if status.starts_with('R') || status.starts_with('C') {
            let previous_path = parts.next().unwrap_or_default().trim().to_string();
            let path = parts.next().unwrap_or_default().trim().to_string();

            files.push(CommitFileChange {
                status,
                display_path: format!("{previous_path} -> {path}"),
                previous_path: Some(previous_path),
                path,
            });
        } else {
            let path = parts.next().unwrap_or_default().trim().to_string();
            files.push(CommitFileChange {
                status,
                display_path: path.clone(),
                previous_path: None,
                path,
            });
        }
    }

    Ok(files)
}

pub fn get_commit_file_diff(repo_path: &str, hash: &str, file_path: &str) -> AppResult<String> {
    validate_repository(repo_path)?;

    let parents = git_command(repo_path, &["show", "-s", "--format=%P", hash])?;
    if parents.trim().is_empty() {
        return git_command(repo_path, &["show", "--format=", hash, "--", file_path]);
    }

    git_command(
        repo_path,
        &["diff", &format!("{hash}^"), hash, "--", file_path],
    )
}

pub fn push_current_branch(repo_path: &str) -> AppResult<()> {
    validate_repository(repo_path)?;
    let _ = git_command(repo_path, &["push"])?;
    Ok(())
}

pub fn push_to_commit(repo_path: &str, branch: &str, hash: &str) -> AppResult<()> {
    validate_repository(repo_path)?;
    let refspec = format!("{hash}:refs/heads/{branch}");
    let _ = git_command(repo_path, &["push", "origin", &refspec])?;
    Ok(())
}
