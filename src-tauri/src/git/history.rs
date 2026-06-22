use std::{collections::HashSet, path::Path};

use crate::{
    error::AppResult,
    models::{CommitFileChange, CommitHistoryFilters, CommitHistoryPage, CommitMeta},
};

use super::{
    command::{git_output_bytes, git_text, git_trimmed},
    parse::{
        apply_commit_file_numstat, parse_commit_files, parse_commit_history, parse_shortstat,
        parse_unpushed_hashes,
    },
    push::safe_unpushed_hashes,
    repository::{branch_status_for_path, resolve_repository_path},
};

#[derive(Debug, Default)]
struct NormalizedCommitHistoryFilters {
    query: Option<String>,
    author: Option<String>,
    file_path: Option<String>,
    message: Option<String>,
}

fn normalize_filter_value(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
}

fn normalize_history_filters(
    filters: Option<CommitHistoryFilters>,
) -> NormalizedCommitHistoryFilters {
    let filters = filters.unwrap_or_default();

    NormalizedCommitHistoryFilters {
        query: normalize_filter_value(filters.query),
        author: normalize_filter_value(filters.author),
        file_path: normalize_filter_value(filters.file_path),
        message: normalize_filter_value(filters.message),
    }
}

impl NormalizedCommitHistoryFilters {
    fn has_filters(&self) -> bool {
        self.has_commit_metadata_filters() || self.file_path.is_some()
    }

    fn has_commit_metadata_filters(&self) -> bool {
        self.query.is_some() || self.author.is_some() || self.message.is_some()
    }
}

fn contains_normalized(value: &str, needle: &str) -> bool {
    value.to_ascii_lowercase().contains(needle)
}

fn git_basic_regex_literal(value: &str) -> String {
    value.chars().fold(String::new(), |mut escaped, character| {
        if matches!(character, '.' | '^' | '$' | '*' | '[' | ']' | '\\') {
            escaped.push('\\');
        }
        escaped.push(character);
        escaped
    })
}

fn filtered_history_log_args(filters: &NormalizedCommitHistoryFilters) -> Vec<String> {
    let mut args = vec![
        "log".to_string(),
        "--topo-order".to_string(),
        "--date=iso-strict".to_string(),
        "--decorate=short".to_string(),
        "--pretty=format:%H%x1f%h%x1f%s%x1f%an%x1f%ae%x1f%cI%x1f%D%x1f%P%x1f%B%x1e".to_string(),
    ];

    if filters.author.is_some() || filters.message.is_some() {
        args.push("--regexp-ignore-case".to_string());
    }

    if let Some(author) = filters.author.as_deref() {
        args.push(format!("--author={}", git_basic_regex_literal(author)));
    }

    if let Some(message) = filters.message.as_deref() {
        args.push(format!("--grep={}", git_basic_regex_literal(message)));
    }

    args.push("HEAD".to_string());
    args
}

fn commit_record_matches_metadata(record: &str, filters: &NormalizedCommitHistoryFilters) -> bool {
    if !filters.has_commit_metadata_filters() {
        return true;
    }

    let fields: Vec<&str> = record.split('\u{1f}').collect();
    if fields.len() < 9 {
        return false;
    }

    if let Some(query) = filters.query.as_deref() {
        let hash = fields[0];
        let short_hash = fields[1];
        let summary = fields[2];
        let author_name = fields[3];
        let author_email = fields[4];
        let decorations = fields[6];
        let body = fields[8];

        if !contains_normalized(hash, query)
            && !contains_normalized(short_hash, query)
            && !contains_normalized(summary, query)
            && !contains_normalized(body, query)
            && !contains_normalized(author_name, query)
            && !contains_normalized(author_email, query)
            && !contains_normalized(decorations, query)
        {
            return false;
        }
    }

    if let Some(message) = filters.message.as_deref() {
        let summary = fields[2];
        let body = fields[8];

        if !contains_normalized(summary, message) && !contains_normalized(body, message) {
            return false;
        }
    }

    if let Some(author) = filters.author.as_deref() {
        let author_name = fields[3];
        let author_email = fields[4];

        if !contains_normalized(author_name, author) && !contains_normalized(author_email, author) {
            return false;
        }
    }

    true
}

fn commit_record_hash(record: &str) -> Option<&str> {
    record
        .split('\u{1f}')
        .next()
        .map(str::trim)
        .filter(|hash| !hash.is_empty())
}

fn commit_matches_file_path(
    repo_path: &Path,
    hash: &str,
    file_path_filter: &str,
) -> AppResult<bool> {
    let files = commit_files_for_resolved_path(repo_path, hash)?;

    Ok(files.iter().any(|file| {
        contains_normalized(&file.path, file_path_filter)
            || file
                .previous_path
                .as_deref()
                .is_some_and(|previous_path| contains_normalized(previous_path, file_path_filter))
            || contains_normalized(&file.display_path, file_path_filter)
    }))
}

fn commit_record_matches_filters(
    repo_path: &Path,
    record: &str,
    filters: &NormalizedCommitHistoryFilters,
) -> AppResult<bool> {
    if !commit_record_matches_metadata(record, filters) {
        return Ok(false);
    }

    let Some(file_path_filter) = filters.file_path.as_deref() else {
        return Ok(true);
    };
    let Some(hash) = commit_record_hash(record) else {
        return Ok(false);
    };

    commit_matches_file_path(repo_path, hash, file_path_filter)
}

pub(super) fn unpushed_hashes(repo_path: &Path, upstream: &str) -> AppResult<HashSet<String>> {
    let range = format!("{upstream}..HEAD");
    let output = git_trimmed(repo_path, &["rev-list", &range])?;
    Ok(parse_unpushed_hashes(&output))
}

pub fn get_commit_history(
    repo_path: &str,
    skip: usize,
    limit: usize,
    filters: Option<CommitHistoryFilters>,
) -> AppResult<CommitHistoryPage> {
    let repo_path = resolve_repository_path(repo_path)?;
    let filters = normalize_history_filters(filters);
    let branch_status = branch_status_for_path(&repo_path)?;
    let (unpushed, safe_push_targets, unsafe_push_reason) = match branch_status.upstream.as_deref()
    {
        Some(upstream) => {
            let unpushed = unpushed_hashes(&repo_path, upstream)?;
            let safe_push_targets = if branch_status.behind_count > 0 {
                HashSet::new()
            } else {
                safe_unpushed_hashes(&repo_path, upstream)?
            };
            let unsafe_push_reason = if branch_status.behind_count > 0 {
                branch_status.disabled_reason.as_deref()
            } else {
                None
            };
            (unpushed, safe_push_targets, unsafe_push_reason)
        }
        _ => (HashSet::new(), HashSet::new(), None),
    };

    let (items, item_count, has_more, total_count) = if filters.has_filters() {
        let log_args = filtered_history_log_args(&filters);
        let log_arg_refs: Vec<&str> = log_args.iter().map(String::as_str).collect();
        let output = git_trimmed(&repo_path, &log_arg_refs)?;
        let mut matched_records: Vec<&str> = Vec::new();
        for record in output
            .split('\u{1e}')
            .filter(|record| !record.trim().is_empty())
        {
            if commit_record_matches_filters(&repo_path, record, &filters)? {
                matched_records.push(record);
            }
        }
        let total_count = matched_records.len();
        let page_records: Vec<&str> = matched_records
            .iter()
            .skip(skip)
            .take(limit)
            .copied()
            .collect();
        let page_output = if page_records.is_empty() {
            String::new()
        } else {
            let mut output = page_records.join("\u{1e}");
            output.push('\u{1e}');
            output
        };
        let items = parse_commit_history(
            &page_output,
            &unpushed,
            &safe_push_targets,
            unsafe_push_reason,
        );
        let item_count = items.len();
        let has_more = skip + item_count < total_count;

        (items, item_count, has_more, total_count)
    } else {
        let query_limit = limit.saturating_add(1);
        let output = git_trimmed(
            &repo_path,
            &[
                "log",
                "--topo-order",
                "--skip",
                &skip.to_string(),
                "-n",
                &query_limit.to_string(),
                "--date=iso-strict",
                "--decorate=short",
                "--pretty=format:%H%x1f%h%x1f%s%x1f%an%x1f%ae%x1f%cI%x1f%D%x1f%P%x1e",
                "HEAD",
            ],
        )?;

        let mut items =
            parse_commit_history(&output, &unpushed, &safe_push_targets, unsafe_push_reason);
        let has_more = items.len() > limit;
        if has_more {
            items.truncate(limit);
        }
        let item_count = items.len();
        let total_count = skip + item_count + usize::from(has_more);

        (items, item_count, has_more, total_count)
    };

    Ok(CommitHistoryPage {
        items,
        next_skip: skip + item_count,
        has_more,
        total_count,
        unpushed_count: unpushed.len(),
        safe_unpushed_count: safe_push_targets.len(),
    })
}

pub fn get_commit_files(repo_path: &str, hash: &str) -> AppResult<Vec<CommitFileChange>> {
    let repo_path = resolve_repository_path(repo_path)?;
    commit_files_for_resolved_path(&repo_path, hash)
}

fn commit_files_for_resolved_path(
    repo_path: &Path,
    hash: &str,
) -> AppResult<Vec<CommitFileChange>> {
    let output = git_output_bytes(
        repo_path,
        &[
            "show",
            "--find-renames",
            "--name-status",
            "-z",
            "--format=",
            hash,
        ],
    )?;
    let mut files = parse_commit_files(&output);
    let numstat_output = git_output_bytes(
        repo_path,
        &[
            "show",
            "--find-renames",
            "--numstat",
            "-z",
            "--format=",
            hash,
        ],
    )?;
    apply_commit_file_numstat(&mut files, &numstat_output);
    Ok(files)
}

pub fn get_commit_meta(repo_path: &str, hash: &str) -> AppResult<CommitMeta> {
    let repo_path = resolve_repository_path(repo_path)?;
    let output = git_text(
        &repo_path,
        &["show", "--shortstat", "--format=%b%x1e", hash],
    )?;
    let mut sections = output.splitn(2, '\u{1e}');
    let body = sections.next().unwrap_or_default().trim().to_string();
    let shortstat = sections.next().unwrap_or_default();
    let (additions, deletions) = parse_shortstat(shortstat);

    Ok(CommitMeta {
        body,
        additions,
        deletions,
    })
}
