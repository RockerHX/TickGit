use std::{collections::HashSet, path::Path};

use crate::{
    error::AppResult,
    models::{
        BranchStatus, CommitDetails, CommitFileChange, CommitHistoryFilters, CommitHistoryPage,
        CommitMeta, RepositoryOverview,
    },
};

use super::{
    command::{git_output_bytes, git_text, git_trimmed},
    parse::{
        apply_commit_file_numstat, parse_commit_files, parse_commit_history, parse_shortstat,
        parse_unpushed_hashes,
    },
    push::safe_unpushed_hashes_in_push_order,
    repository::{
        ahead_behind, current_branch_name, remote_origin_exists, resolve_repository_path,
        total_ahead_count, upstream_is_origin, upstream_name,
    },
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

fn matching_commit_hashes_for_path_filter(
    repo_path: &Path,
    file_path_filter: &str,
) -> AppResult<HashSet<String>> {
    let output = git_output_bytes(
        repo_path,
        &[
            "log",
            "--topo-order",
            "--find-renames",
            "--pretty=format:%H%x00",
            "--name-status",
            "-z",
            "HEAD",
        ],
    )?;
    let mut hashes = HashSet::new();
    let mut current_hash: Option<String> = None;
    let mut parts = output.split(|byte| *byte == b'\0');

    while let Some(part) = parts.next() {
        if part.is_empty() {
            continue;
        }

        let value = String::from_utf8_lossy(part).to_string();
        let value = value.trim();

        if value.len() == 40 && value.chars().all(|ch| ch.is_ascii_hexdigit()) {
            current_hash = Some(value.to_string());
            continue;
        }

        let status = value;
        if status.starts_with('R') || status.starts_with('C') {
            let previous_path = parts
                .next()
                .map(|value| String::from_utf8_lossy(value).to_string())
                .unwrap_or_default();
            let path = parts
                .next()
                .map(|value| String::from_utf8_lossy(value).to_string())
                .unwrap_or_default();
            let display_path = format!("{previous_path} -> {path}");

            if let Some(hash) = current_hash.as_ref() {
                if contains_normalized(&previous_path, file_path_filter)
                    || contains_normalized(&path, file_path_filter)
                    || contains_normalized(&display_path, file_path_filter)
                {
                    hashes.insert(hash.clone());
                }
            }
            continue;
        }

        let path = parts
            .next()
            .map(|value| String::from_utf8_lossy(value).to_string())
            .unwrap_or_default();

        if let Some(hash) = current_hash.as_ref() {
            if contains_normalized(&path, file_path_filter) {
                hashes.insert(hash.clone());
            }
        }
    }

    Ok(hashes)
}

fn commit_record_matches_filters_with_path_hashes(
    record: &str,
    filters: &NormalizedCommitHistoryFilters,
    path_filter_hashes: Option<&HashSet<String>>,
) -> bool {
    if !commit_record_matches_metadata(record, filters) {
        return false;
    }

    let Some(path_filter_hashes) = path_filter_hashes else {
        return true;
    };
    let Some(hash) = commit_record_hash(record) else {
        return false;
    };

    path_filter_hashes.contains(hash)
}

fn branch_status_and_push_sets(
    repo_path: &Path,
) -> AppResult<(BranchStatus, HashSet<String>, HashSet<String>, Option<String>)> {
    let (branch, detached) = current_branch_name(repo_path)?;
    let has_origin = remote_origin_exists(repo_path);
    let upstream = if detached || !has_origin {
        None
    } else {
        upstream_name(repo_path)
    };

    let (ahead_count, behind_count, unpushed, safe_push_targets) = match upstream.as_deref() {
        Some(upstream) => {
            let (_, behind_count) = ahead_behind(repo_path, upstream)?;
            let ahead_count = total_ahead_count(repo_path, upstream)?;
            let range = format!("{upstream}..HEAD");
            let unpushed_output = git_trimmed(repo_path, &["rev-list", &range])?;
            let unpushed = parse_unpushed_hashes(&unpushed_output);
            let safe_push_targets = if behind_count > 0 {
                HashSet::new()
            } else {
                safe_unpushed_hashes_in_push_order(repo_path, upstream)?
                    .into_iter()
                    .collect()
            };
            (ahead_count, behind_count, unpushed, safe_push_targets)
        }
        None => (0, 0, HashSet::new(), HashSet::new()),
    };

    let upstream_uses_origin = upstream.as_deref().is_some_and(upstream_is_origin);
    let push_available = !detached && has_origin && upstream_uses_origin && behind_count == 0;
    let safe_ahead_count = if behind_count > 0 {
        0
    } else {
        safe_push_targets.len()
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
            Some(super::BRANCH_BEHIND_REMOTE_MESSAGE.to_string()),
            Some("behind_remote".to_string()),
        )
    } else {
        (None, None)
    };
    let unsafe_push_reason = if behind_count > 0 {
        disabled_reason.as_deref().map(ToOwned::to_owned)
    } else {
        None
    };

    Ok((
        BranchStatus {
            branch,
            upstream,
            ahead_count,
            safe_ahead_count,
            behind_count,
            detached,
            push_available,
            disabled_reason,
            disabled_reason_code,
        },
        unpushed,
        safe_push_targets,
        unsafe_push_reason,
    ))
}

fn commit_history_for_resolved_path(
    repo_path: &Path,
    skip: usize,
    limit: usize,
    filters: NormalizedCommitHistoryFilters,
    unpushed: &HashSet<String>,
    safe_push_targets: &HashSet<String>,
    unsafe_push_reason: Option<&str>,
) -> AppResult<CommitHistoryPage> {
    let (items, item_count, has_more, total_count) = if filters.has_filters() {
        let path_filter_hashes = filters
            .file_path
            .as_deref()
            .map(|file_path_filter| matching_commit_hashes_for_path_filter(repo_path, file_path_filter))
            .transpose()?;
        let output = git_trimmed(
            repo_path,
            &[
                "log",
                "--topo-order",
                "--date=iso-strict",
                "--decorate=short",
                "--pretty=format:%H%x1f%h%x1f%s%x1f%an%x1f%ae%x1f%cI%x1f%D%x1f%P%x1f%B%x1e",
                "HEAD",
            ],
        )?;
        let mut matched_records: Vec<&str> = Vec::new();
        for record in output
            .split('\u{1e}')
            .filter(|record| !record.trim().is_empty())
        {
            if commit_record_matches_filters_with_path_hashes(
                record,
                &filters,
                path_filter_hashes.as_ref(),
            ) {
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
            unpushed,
            safe_push_targets,
            unsafe_push_reason,
        );
        let item_count = items.len();
        let has_more = skip + item_count < total_count;

        (items, item_count, has_more, total_count)
    } else {
        let total_count = git_trimmed(repo_path, &["rev-list", "--count", "HEAD"])?
            .parse::<usize>()
            .unwrap_or(0);
        let output = git_trimmed(
            repo_path,
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

        let items = parse_commit_history(&output, unpushed, safe_push_targets, unsafe_push_reason);
        let item_count = items.len();
        let has_more = skip + item_count < total_count;

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

pub fn get_commit_history(
    repo_path: &str,
    skip: usize,
    limit: usize,
    filters: Option<CommitHistoryFilters>,
) -> AppResult<CommitHistoryPage> {
    let repo_path = resolve_repository_path(repo_path)?;
    let filters = normalize_history_filters(filters);
    let (_branch_status, unpushed, safe_push_targets, unsafe_push_reason) =
        branch_status_and_push_sets(&repo_path)?;
    commit_history_for_resolved_path(
        &repo_path,
        skip,
        limit,
        filters,
        &unpushed,
        &safe_push_targets,
        unsafe_push_reason.as_deref(),
    )
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
    commit_meta_for_resolved_path(&repo_path, hash)
}

fn commit_meta_for_resolved_path(repo_path: &Path, hash: &str) -> AppResult<CommitMeta> {
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

pub fn get_commit_details(repo_path: &str, hash: &str) -> AppResult<CommitDetails> {
    let repo_path = resolve_repository_path(repo_path)?;
    Ok(CommitDetails {
        meta: commit_meta_for_resolved_path(&repo_path, hash)?,
        files: commit_files_for_resolved_path(&repo_path, hash)?,
    })
}

pub fn get_repository_overview(
    repo_path: &str,
    skip: usize,
    limit: usize,
    filters: Option<CommitHistoryFilters>,
) -> AppResult<RepositoryOverview> {
    let repo_path = resolve_repository_path(repo_path)?;
    let filters = normalize_history_filters(filters);
    let (branch_status, unpushed, safe_push_targets, unsafe_push_reason) =
        branch_status_and_push_sets(&repo_path)?;
    let branches = super::repository::list_local_branches_for_path(&repo_path)?;
    let history = commit_history_for_resolved_path(
        &repo_path,
        skip,
        limit,
        filters,
        &unpushed,
        &safe_push_targets,
        unsafe_push_reason.as_deref(),
    )?;

    Ok(RepositoryOverview {
        branch_status,
        branches,
        history,
    })
}
