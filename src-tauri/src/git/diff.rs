use std::path::Path;

use crate::{error::AppResult, models::CommitFileDiffResult};

use super::{
    command::{git_text, git_text_allow_exit_code, git_trimmed, git_trimmed_allow_exit_code},
    repository::resolve_repository_path,
};

// Git 约定的空树对象。初始提交没有 parent 时，使用它与目标提交做 diff，
// 才能和普通提交一样统一走 diff 参数（例如 -w 忽略空白）逻辑。
const EMPTY_TREE_HASH: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const MAX_DIFF_BYTES: usize = 1024 * 1024;
const MAX_DIFF_LINES: usize = 5000;

#[derive(Default)]
struct DiffStats {
    is_binary: bool,
    line_count: usize,
}

fn is_image_path(file_path: &str) -> bool {
    Path::new(file_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico" | "avif"
            )
        })
        .unwrap_or(false)
}

fn parse_numstat(output: &str) -> DiffStats {
    let mut stats = DiffStats::default();

    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let mut fields = line.split('\t');
        let additions = fields.next().unwrap_or_default();
        let deletions = fields.next().unwrap_or_default();

        if additions == "-" || deletions == "-" {
            stats.is_binary = true;
            continue;
        }

        stats.line_count += additions.parse::<usize>().unwrap_or_default();
        stats.line_count += deletions.parse::<usize>().unwrap_or_default();
    }

    stats
}

fn diff_base_and_pathspecs<'a>(
    repo_path: &Path,
    hash: &'a str,
    file_path: &'a str,
    previous_path: Option<&'a str>,
) -> AppResult<(String, Vec<&'a str>)> {
    let parents = git_trimmed(repo_path, &["show", "-s", "--format=%P", hash])?;
    let base = if parents.trim().is_empty() {
        EMPTY_TREE_HASH.to_string()
    } else {
        format!("{hash}^")
    };

    let mut pathspecs = Vec::new();

    if let Some(previous_path) = previous_path
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != file_path)
    {
        pathspecs.push(previous_path);
    }

    pathspecs.push(file_path);

    Ok((base, pathspecs))
}

fn build_diff_args<'a>(
    base: &'a str,
    hash: &'a str,
    pathspecs: &[&'a str],
    ignore_whitespace: bool,
    include_numstat: bool,
) -> Vec<&'a str> {
    let mut args = vec!["diff"];

    if include_numstat {
        args.push("--numstat");
    }

    if ignore_whitespace {
        args.push("-w");
    }

    args.extend(["--find-renames", base, hash, "--"]);
    args.extend(pathspecs.iter().copied());
    args
}

pub(super) fn diff_result_from_git_args(
    repo_path: &Path,
    file_path: &str,
    numstat_args: &[&str],
    diff_args: &[&str],
    allowed_diff_exit_code: Option<i32>,
) -> AppResult<CommitFileDiffResult> {
    let is_image = is_image_path(file_path);
    let numstat = match allowed_diff_exit_code {
        Some(code) => git_trimmed_allow_exit_code(repo_path, numstat_args, code)?,
        None => git_trimmed(repo_path, numstat_args)?,
    };
    let stats = parse_numstat(&numstat);

    if stats.is_binary || is_image {
        return Ok(CommitFileDiffResult {
            text: String::new(),
            is_binary: stats.is_binary,
            is_image,
            is_too_large: false,
            truncated: false,
            byte_count: 0,
            line_count: stats.line_count,
            old_image_data_url: None,
            new_image_data_url: None,
        });
    }

    if stats.line_count > MAX_DIFF_LINES {
        return Ok(CommitFileDiffResult {
            text: String::new(),
            is_binary: false,
            is_image: false,
            is_too_large: true,
            truncated: true,
            byte_count: 0,
            line_count: stats.line_count,
            old_image_data_url: None,
            new_image_data_url: None,
        });
    }

    let text = match allowed_diff_exit_code {
        Some(code) => git_text_allow_exit_code(repo_path, diff_args, code)?,
        None => git_text(repo_path, diff_args)?,
    };
    let byte_count = text.len();

    if byte_count > MAX_DIFF_BYTES {
        return Ok(CommitFileDiffResult {
            text: String::new(),
            is_binary: false,
            is_image: false,
            is_too_large: true,
            truncated: true,
            byte_count,
            line_count: stats.line_count,
            old_image_data_url: None,
            new_image_data_url: None,
        });
    }

    Ok(CommitFileDiffResult {
        text,
        is_binary: false,
        is_image: false,
        is_too_large: false,
        truncated: false,
        byte_count,
        line_count: stats.line_count,
        old_image_data_url: None,
        new_image_data_url: None,
    })
}

pub fn get_commit_file_diff(
    repo_path: &str,
    hash: &str,
    file_path: &str,
    previous_path: Option<&str>,
    ignore_whitespace: bool,
) -> AppResult<CommitFileDiffResult> {
    let repo_path = resolve_repository_path(repo_path)?;
    let (base, pathspecs) = diff_base_and_pathspecs(&repo_path, hash, file_path, previous_path)?;
    let numstat_args = build_diff_args(&base, hash, &pathspecs, ignore_whitespace, true);
    let diff_args = build_diff_args(&base, hash, &pathspecs, ignore_whitespace, false);
    diff_result_from_git_args(&repo_path, file_path, &numstat_args, &diff_args, None)
}
