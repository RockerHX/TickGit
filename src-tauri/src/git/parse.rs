use std::collections::{HashMap, HashSet};

use crate::models::{CommitFileChange, CommitListItem};

use super::UNSAFE_PUSH_TARGET_MESSAGE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CommitFileNumstat {
    path: String,
    additions: Option<usize>,
    deletions: Option<usize>,
}

pub(super) fn parse_ahead_behind(counts: &str) -> (usize, usize) {
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

pub(super) fn parse_count(output: &str) -> usize {
    output.trim().parse::<usize>().unwrap_or_default()
}

pub(super) fn parse_unpushed_hashes(output: &str) -> HashSet<String> {
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

pub(super) fn parse_commit_history(
    output: &str,
    unpushed: &HashSet<String>,
    safe_push_targets: &HashSet<String>,
    unsafe_push_reason: Option<&str>,
) -> Vec<CommitListItem> {
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
            let is_pushed = !unpushed.contains(&hash);
            let is_safe_push_target = safe_push_targets.contains(&hash);

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
                is_pushed,
                is_safe_push_target,
                push_blocked_reason: (!is_pushed && !is_safe_push_target).then(|| {
                    unsafe_push_reason
                        .unwrap_or(UNSAFE_PUSH_TARGET_MESSAGE)
                        .to_string()
                }),
                push_blocked_reason_code: (!is_pushed && !is_safe_push_target)
                    .then(|| "unsafe_push_target".to_string()),
                hash,
            })
        })
        .collect()
}

pub(super) fn parse_commit_files(output: &[u8]) -> Vec<CommitFileChange> {
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
                language: infer_file_language(&path),
                additions: None,
                deletions: None,
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
                language: infer_file_language(&path),
                additions: None,
                deletions: None,
                previous_path: None,
                path,
                status,
            });
        }
    }

    files
}

fn infer_file_language(path: &str) -> Option<String> {
    let file_name = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();
    let extension = file_name.rsplit_once('.').map(|(_, value)| value);

    if file_name == "package.json" {
        return Some("json".to_string());
    }

    if file_name.ends_with(".lock")
        || matches!(
            file_name.as_str(),
            "package-lock.json" | "pnpm-lock.yaml" | "bun.lockb"
        )
    {
        return Some("lock".to_string());
    }

    match extension {
        Some("json") => Some("json".to_string()),
        Some("yaml" | "yml") => Some("yaml".to_string()),
        _ => None,
    }
}

fn parse_numstat_counts(additions: &str, deletions: &str) -> (Option<usize>, Option<usize>) {
    match (additions.parse::<usize>(), deletions.parse::<usize>()) {
        (Ok(additions), Ok(deletions)) => (Some(additions), Some(deletions)),
        _ => (None, None),
    }
}

pub(super) fn parse_commit_file_numstat(output: &[u8]) -> Vec<CommitFileNumstat> {
    let mut stats = Vec::new();
    let mut parts = output
        .split(|byte| *byte == b'\0')
        .filter(|part| !part.is_empty());

    while let Some(stat_bytes) = parts.next() {
        let stat = String::from_utf8_lossy(stat_bytes);
        let mut fields = stat.splitn(3, '\t');
        let additions = fields.next().unwrap_or_default();
        let deletions = fields.next().unwrap_or_default();
        let path_field = fields.next().unwrap_or_default();
        let path = if path_field.is_empty() {
            let _previous_path = parts.next();
            parts
                .next()
                .map(|value| String::from_utf8_lossy(value).to_string())
                .unwrap_or_default()
        } else {
            path_field.to_string()
        };

        if path.is_empty() {
            continue;
        }

        let (additions, deletions) = parse_numstat_counts(additions, deletions);
        stats.push(CommitFileNumstat {
            path,
            additions,
            deletions,
        });
    }

    stats
}

pub(super) fn apply_commit_file_numstat(files: &mut [CommitFileChange], output: &[u8]) {
    let stats_by_path = parse_commit_file_numstat(output)
        .into_iter()
        .map(|stats| (stats.path.clone(), stats))
        .collect::<HashMap<_, _>>();

    for file in files {
        if let Some(stats) = stats_by_path.get(&file.path) {
            file.additions = stats.additions;
            file.deletions = stats.deletions;
        }
    }
}

pub(super) fn parse_shortstat(output: &str) -> (usize, usize) {
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

#[cfg(test)]
mod tests {
    use super::{
        apply_commit_file_numstat, parse_ahead_behind, parse_commit_file_numstat,
        parse_commit_files, parse_commit_history, parse_shortstat,
    };
    use std::collections::HashSet;

    #[test]
    fn parses_ahead_behind_counts() {
        assert_eq!(parse_ahead_behind("3\t2"), (2, 3));
        assert_eq!(parse_ahead_behind("0 0"), (0, 0));
        assert_eq!(parse_ahead_behind("bad"), (0, 0));
    }

    #[test]
    fn parses_commit_history_records() {
        let unpushed = HashSet::from([String::from("hash-2")]);
        let safe_push_targets = HashSet::from([String::from("hash-2")]);
        let output = concat!(
            "hash-1\x1fshort-1\x1fInitial commit\x1fAlice\x1falice@example.com\x1f2026-04-25T10:00:00Z\x1ftag: v1.0.0\x1f\x1e",
            "hash-2\x1fshort-2\x1fAdd file\x1fBob\x1fbob@example.com\x1f2026-04-25T11:00:00Z\x1fHEAD -> main, tag: v1.1.0, origin/main, tag: latest\x1fhash-1\x1e",
        );

        let items = parse_commit_history(output, &unpushed, &safe_push_targets, None);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].hash, "hash-1");
        assert!(items[0].is_pushed);
        assert!(!items[0].is_safe_push_target);
        assert_eq!(items[0].push_blocked_reason, None);
        assert_eq!(items[0].tags, vec!["v1.0.0"]);
        assert_eq!(items[0].parents, Vec::<String>::new());
        assert_eq!(items[1].hash, "hash-2");
        assert!(!items[1].is_pushed);
        assert!(items[1].is_safe_push_target);
        assert_eq!(items[1].push_blocked_reason, None);
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
    fn infers_commit_file_language() {
        let output = b"M\0package.json\0M\0config.yaml\0M\0pnpm-lock.yaml\0M\0src/main.rs\0";

        let files = parse_commit_files(output);

        assert_eq!(files[0].language.as_deref(), Some("json"));
        assert_eq!(files[1].language.as_deref(), Some("yaml"));
        assert_eq!(files[2].language.as_deref(), Some("lock"));
        assert_eq!(files[3].language, None);
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
    fn parses_commit_file_numstat() {
        let output = b"3\t1\tsrc/main.rs\0-\t-\tassets/logo.png\0";

        let stats = parse_commit_file_numstat(output);

        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].path, "src/main.rs");
        assert_eq!(stats[0].additions, Some(3));
        assert_eq!(stats[0].deletions, Some(1));
        assert_eq!(stats[1].path, "assets/logo.png");
        assert_eq!(stats[1].additions, None);
        assert_eq!(stats[1].deletions, None);
    }

    #[test]
    fn parses_commit_file_numstat_for_renames_with_tabs_in_paths() {
        let output = b"2\t0\t\0old\tname.txt\0new\tname.txt\0";

        let stats = parse_commit_file_numstat(output);

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].path, "new\tname.txt");
        assert_eq!(stats[0].additions, Some(2));
        assert_eq!(stats[0].deletions, Some(0));
    }

    #[test]
    fn applies_commit_file_numstat_to_matching_files() {
        let output = b"M\0src/main.rs\0R100\0old\tname.txt\0new\tname.txt\0";
        let numstat = b"3\t1\tsrc/main.rs\0\x32\t0\t\0old\tname.txt\0new\tname.txt\0";
        let mut files = parse_commit_files(output);

        apply_commit_file_numstat(&mut files, numstat);

        assert_eq!(files[0].additions, Some(3));
        assert_eq!(files[0].deletions, Some(1));
        assert_eq!(files[1].path, "new\tname.txt");
        assert_eq!(files[1].additions, Some(2));
        assert_eq!(files[1].deletions, Some(0));
    }

    #[test]
    fn parses_shortstat_counts() {
        assert_eq!(
            parse_shortstat(" 1 file changed, 3 insertions(+), 2 deletions(-)"),
            (3, 2)
        );
        assert_eq!(parse_shortstat(" 1 file changed, 4 insertions(+)"), (4, 0));
        assert_eq!(parse_shortstat(" 1 file changed, 7 deletions(-)"), (0, 7));
        assert_eq!(parse_shortstat(""), (0, 0));
    }
}
