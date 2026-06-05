use std::{collections::HashSet, path::Path};

use crate::{
    error::AppResult,
    models::{CommitFileChange, CommitHistoryFilters, CommitHistoryPage, CommitMeta},
};

use super::{
    command::{git_output_bytes, git_text, git_trimmed},
    parse::{parse_commit_files, parse_commit_history, parse_shortstat, parse_unpushed_hashes},
    push::safe_unpushed_hashes,
    repository::{branch_status_for_path, resolve_repository_path},
};

#[derive(Debug, Default)]
struct NormalizedCommitHistoryFilters {
    query: Option<String>,
    author: Option<String>,
    file_path: Option<String>,
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
    }
}

impl NormalizedCommitHistoryFilters {
    fn has_commit_metadata_filters(&self) -> bool {
        self.query.is_some() || self.author.is_some()
    }
}

fn contains_normalized(value: &str, needle: &str) -> bool {
    value.to_ascii_lowercase().contains(needle)
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
        let summary = fields[2];
        let body = fields[8];

        if !contains_normalized(summary, query) && !contains_normalized(body, query) {
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

    let (items, item_count, has_more) = if filters.has_commit_metadata_filters() {
        let output = git_trimmed(
            &repo_path,
            &[
                "log",
                "--topo-order",
                "--date=iso-strict",
                "--decorate=short",
                "--pretty=format:%H%x1f%h%x1f%s%x1f%an%x1f%ae%x1f%cI%x1f%D%x1f%P%x1f%B%x1e",
                "HEAD",
            ],
        )?;
        let matched_records: Vec<&str> = output
            .split('\u{1e}')
            .filter(|record| !record.trim().is_empty())
            .filter(|record| commit_record_matches_metadata(record, &filters))
            .collect();
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
        let has_more = skip + item_count < matched_records.len();

        (items, item_count, has_more)
    } else {
        let output = git_trimmed(
            &repo_path,
            &[
                "log",
                "--topo-order",
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

        let items =
            parse_commit_history(&output, &unpushed, &safe_push_targets, unsafe_push_reason);
        let item_count = items.len();
        let has_more = item_count == limit;

        (items, item_count, has_more)
    };

    Ok(CommitHistoryPage {
        items,
        next_skip: skip + item_count,
        has_more,
        unpushed_count: unpushed.len(),
        safe_unpushed_count: safe_push_targets.len(),
    })
}

pub fn get_commit_files(repo_path: &str, hash: &str) -> AppResult<Vec<CommitFileChange>> {
    let repo_path = resolve_repository_path(repo_path)?;
    let output = git_output_bytes(
        &repo_path,
        &[
            "show",
            "--find-renames",
            "--name-status",
            "-z",
            "--format=",
            hash,
        ],
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
