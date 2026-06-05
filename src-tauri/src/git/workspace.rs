use std::path::Path;

use crate::{
    error::{AppError, AppResult},
    models::{
        CommitCreated, CommitFileDiffResult, WorkspaceChangeKind, WorkspaceChangeSection,
        WorkspaceFileChange, WorkspaceStatus,
    },
};

use super::{
    command::{git_output_bytes, git_run, git_trimmed},
    diff::{
        diff_result_from_git_args, git_image_data_url, index_image_data_url, is_image_path,
        working_tree_image_data_url,
    },
    repository::resolve_repository_path,
};

fn workspace_file_path(file_path: &str) -> AppResult<&str> {
    if file_path.is_empty() {
        return Err(AppError::new("invalid_file_path", "文件路径不能为空"));
    }

    if file_path.contains('\0') {
        return Err(AppError::new("invalid_file_path", "文件路径不能包含空字符"));
    }

    Ok(file_path)
}

fn kind_from_status(status: char, untracked: bool) -> WorkspaceChangeKind {
    if untracked {
        return WorkspaceChangeKind::Untracked;
    }

    match status {
        'M' => WorkspaceChangeKind::Modified,
        'A' => WorkspaceChangeKind::Added,
        'D' => WorkspaceChangeKind::Deleted,
        'R' => WorkspaceChangeKind::Renamed,
        'C' => WorkspaceChangeKind::Copied,
        _ => WorkspaceChangeKind::Unknown,
    }
}

fn display_path(path: &str, previous_path: Option<&str>) -> String {
    previous_path
        .map(|previous_path| format!("{previous_path} -> {path}"))
        .unwrap_or_else(|| path.to_string())
}

fn workspace_change(
    section: WorkspaceChangeSection,
    status: char,
    path: &str,
    previous_path: Option<&str>,
) -> WorkspaceFileChange {
    let untracked = status == '?';

    WorkspaceFileChange {
        section,
        kind: kind_from_status(status, untracked),
        status: if untracked {
            "??".to_string()
        } else {
            status.to_string()
        },
        path: path.to_string(),
        previous_path: previous_path.map(ToOwned::to_owned),
        display_path: display_path(path, previous_path),
    }
}

fn parse_workspace_status(output: &[u8]) -> WorkspaceStatus {
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut parts = output
        .split(|byte| *byte == b'\0')
        .filter(|part| !part.is_empty());

    while let Some(record) = parts.next() {
        if record.len() < 4 {
            continue;
        }

        let index_status = record[0] as char;
        let worktree_status = record[1] as char;
        let path = String::from_utf8_lossy(&record[3..]).to_string();
        let rename_previous_path = if matches!(index_status, 'R' | 'C') {
            parts
                .next()
                .map(|value| String::from_utf8_lossy(value).to_string())
        } else {
            None
        };

        if index_status == '?' && worktree_status == '?' {
            unstaged.push(workspace_change(
                WorkspaceChangeSection::Unstaged,
                '?',
                &path,
                None,
            ));
            continue;
        }

        if index_status != ' ' && index_status != '!' {
            staged.push(workspace_change(
                WorkspaceChangeSection::Staged,
                index_status,
                &path,
                rename_previous_path.as_deref(),
            ));
        }

        if worktree_status != ' ' && worktree_status != '!' {
            unstaged.push(workspace_change(
                WorkspaceChangeSection::Unstaged,
                worktree_status,
                &path,
                None,
            ));
        }
    }

    WorkspaceStatus { staged, unstaged }
}

fn pathspecs<'a>(file_path: &'a str, previous_path: Option<&'a str>) -> Vec<&'a str> {
    let mut pathspecs = Vec::new();

    if let Some(previous_path) = previous_path
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != file_path)
    {
        pathspecs.push(previous_path);
    }

    pathspecs.push(file_path);
    pathspecs
}

fn build_workspace_diff_args<'a>(
    section: &WorkspaceChangeSection,
    file_path: &'a str,
    previous_path: Option<&'a str>,
    ignore_whitespace: bool,
    include_numstat: bool,
) -> Vec<&'a str> {
    let mut args = vec!["diff"];

    if matches!(section, WorkspaceChangeSection::Staged) {
        args.push("--cached");
    }

    if include_numstat {
        args.push("--numstat");
    }

    if ignore_whitespace {
        args.push("-w");
    }

    args.push("--find-renames");
    args.push("--");
    args.extend(pathspecs(file_path, previous_path));
    args
}

fn build_untracked_diff_args(
    file_path: &str,
    ignore_whitespace: bool,
    include_numstat: bool,
) -> Vec<&str> {
    let mut args = vec!["diff", "--no-index"];

    if include_numstat {
        args.push("--numstat");
    }

    if ignore_whitespace {
        args.push("-w");
    }

    args.extend(["--", "/dev/null", file_path]);
    args
}

fn is_untracked_file(repo_path: &Path, file_path: &str) -> AppResult<bool> {
    let output = git_trimmed(
        repo_path,
        &[
            "ls-files",
            "--others",
            "--exclude-standard",
            "--",
            file_path,
        ],
    )?;

    Ok(output.lines().any(|line| line == file_path))
}

pub fn get_workspace_status(repo_path: &str) -> AppResult<WorkspaceStatus> {
    let repo_path = resolve_repository_path(repo_path)?;
    let output = git_output_bytes(
        &repo_path,
        &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
    )?;

    Ok(parse_workspace_status(&output))
}

pub fn get_workspace_file_diff(
    repo_path: &str,
    section: WorkspaceChangeSection,
    file_path: &str,
    previous_path: Option<&str>,
    ignore_whitespace: bool,
) -> AppResult<CommitFileDiffResult> {
    let repo_path = resolve_repository_path(repo_path)?;
    let file_path = workspace_file_path(file_path)?;
    let is_untracked = matches!(section, WorkspaceChangeSection::Unstaged)
        && is_untracked_file(&repo_path, file_path)?;

    if is_untracked {
        let numstat_args = build_untracked_diff_args(file_path, ignore_whitespace, true);
        let diff_args = build_untracked_diff_args(file_path, ignore_whitespace, false);
        let image_data_urls = is_image_path(file_path)
            .then(|| Ok::<_, AppError>((None, working_tree_image_data_url(&repo_path, file_path)?)))
            .transpose()?;
        return diff_result_from_git_args(
            &repo_path,
            file_path,
            &numstat_args,
            &diff_args,
            Some(1),
            image_data_urls,
        );
    }

    let numstat_args =
        build_workspace_diff_args(&section, file_path, previous_path, ignore_whitespace, true);
    let diff_args =
        build_workspace_diff_args(&section, file_path, previous_path, ignore_whitespace, false);
    let image_data_urls = is_image_path(file_path)
        .then(|| {
            let old_path = previous_path.unwrap_or(file_path);
            match section {
                WorkspaceChangeSection::Staged => Ok::<_, AppError>((
                    git_image_data_url(&repo_path, "HEAD", old_path)?,
                    index_image_data_url(&repo_path, file_path)?,
                )),
                WorkspaceChangeSection::Unstaged => Ok::<_, AppError>((
                    index_image_data_url(&repo_path, old_path)?,
                    working_tree_image_data_url(&repo_path, file_path)?,
                )),
            }
        })
        .transpose()?;

    diff_result_from_git_args(
        &repo_path,
        file_path,
        &numstat_args,
        &diff_args,
        None,
        image_data_urls,
    )
}

pub fn stage_workspace_file(repo_path: &str, file_path: &str) -> AppResult<()> {
    let repo_path = resolve_repository_path(repo_path)?;
    let file_path = workspace_file_path(file_path)?;
    git_run(&repo_path, &["add", "--", file_path])
}

pub fn unstage_workspace_file(repo_path: &str, file_path: &str) -> AppResult<()> {
    let repo_path = resolve_repository_path(repo_path)?;
    let file_path = workspace_file_path(file_path)?;
    git_run(&repo_path, &["restore", "--staged", "--", file_path])
}

fn staged_change_count(repo_path: &Path) -> AppResult<usize> {
    let output = git_trimmed(repo_path, &["diff", "--cached", "--name-only", "--"])?;
    Ok(output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count())
}

pub fn create_commit(repo_path: &str, message: &str) -> AppResult<CommitCreated> {
    let repo_path = resolve_repository_path(repo_path)?;
    let message = message.trim();

    if message.is_empty() {
        return Err(AppError::new("invalid_commit_message", "提交信息不能为空"));
    }

    if staged_change_count(&repo_path)? == 0 {
        return Err(AppError::new(
            "empty_commit",
            "没有已暂存的变更，无法创建提交",
        ));
    }

    git_run(&repo_path, &["commit", "-m", message])?;

    let hash = git_trimmed(&repo_path, &["rev-parse", "HEAD"])?;
    let short_hash = git_trimmed(&repo_path, &["rev-parse", "--short", "HEAD"])?;
    let summary = git_trimmed(&repo_path, &["show", "-s", "--format=%s", "HEAD"])?;

    Ok(CommitCreated {
        hash,
        short_hash,
        summary,
    })
}
