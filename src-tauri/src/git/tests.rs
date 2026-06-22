use super::repository::branch_status_for_path;
use super::{
    checkout_branch, get_commit_file_diff, get_commit_history, get_commit_meta,
    get_repository_revision, get_step_push_plan, list_local_branches, push_current_branch_checked,
    push_to_commit, refresh_remote_tracking, resolve_repository_path, validate_current_branch,
    validate_step_push_hashes, BRANCH_BEHIND_REMOTE_MESSAGE, BRANCH_MISMATCH_MESSAGE,
    UNSAFE_PUSH_TARGET_MESSAGE,
};
use crate::{error::AppError, models::CommitHistoryFilters};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_TEST_ID: AtomicUsize = AtomicUsize::new(1);

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(prefix: &str) -> Self {
        let suffix = NEXT_TEST_ID.fetch_add(1, Ordering::SeqCst);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let path = env::temp_dir().join(format!(
            "tickgit-{prefix}-{}-{timestamp}-{suffix}",
            std::process::id()
        ));

        fs::create_dir_all(&path).expect("create temp test directory");
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn run_git(path: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .expect("run git command");

    if output.status.success() {
        return String::from_utf8_lossy(&output.stdout).trim().to_string();
    }

    panic!("{}", String::from_utf8_lossy(&output.stderr));
}

fn init_repo() -> TestDir {
    let repo = TestDir::new("repo");
    run_git(&repo.path, &["init"]);
    run_git(&repo.path, &["config", "user.name", "TickGit Tests"]);
    run_git(
        &repo.path,
        &["config", "user.email", "tickgit-tests@example.com"],
    );
    repo
}

fn init_bare_repo() -> TestDir {
    let repo = TestDir::new("bare-repo");
    run_git(&repo.path, &["init", "--bare"]);
    repo
}

fn write_file(path: &Path, relative_path: &str, content: &str) {
    let target = path.join(relative_path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).expect("create parent directory");
    }
    fs::write(target, content).expect("write file");
}

fn commit_file(path: &Path, relative_path: &str, content: &str, message: &str) -> String {
    write_file(path, relative_path, content);
    run_git(path, &["add", relative_path]);
    run_git(path, &["commit", "--no-gpg-sign", "-m", message]);
    run_git(path, &["rev-parse", "HEAD"])
}

fn commit_file_with_body(
    path: &Path,
    relative_path: &str,
    content: &str,
    message: &str,
    body: &str,
) -> String {
    write_file(path, relative_path, content);
    run_git(path, &["add", relative_path]);
    run_git(
        path,
        &["commit", "--no-gpg-sign", "-m", message, "-m", body],
    );
    run_git(path, &["rev-parse", "HEAD"])
}

fn svg_content(label: &str) -> String {
    format!(r#"<svg xmlns="http://www.w3.org/2000/svg"><text>{label}</text></svg>"#)
}

fn assert_svg_data_url(value: Option<&String>) {
    assert!(value
        .expect("svg data url")
        .starts_with("data:image/svg+xml;base64,"));
}

fn current_test_branch(path: &Path) -> String {
    run_git(path, &["branch", "--show-current"])
}

fn clone_repo(source: &Path, prefix: &str) -> TestDir {
    let repo = TestDir::new(prefix);
    run_git(
        source.parent().expect("clone source parent"),
        &[
            "clone",
            source.to_str().expect("clone source path"),
            repo.path.to_str().expect("clone target path"),
        ],
    );
    run_git(&repo.path, &["config", "user.name", "TickGit Tests"]);
    run_git(
        &repo.path,
        &["config", "user.email", "tickgit-tests@example.com"],
    );
    repo
}

fn assert_app_error(error: AppError, code: &str, message: &str) {
    assert_eq!(error.code, code);
    assert_eq!(error.message, message);
}

#[test]
fn resolves_valid_repository_path() {
    let repo = init_repo();
    let resolved = resolve_repository_path(repo.path.to_string_lossy().as_ref()).unwrap();
    assert_eq!(resolved, repo.path.canonicalize().unwrap());
}

#[test]
fn rejects_missing_repository_path() {
    let error = resolve_repository_path("/tmp/tickgit-missing-repo-path").unwrap_err();
    assert_app_error(error, "invalid_repository", "仓库路径不存在");
}

#[test]
fn rejects_non_git_directory() {
    let dir = TestDir::new("plain-dir");
    let error = resolve_repository_path(dir.path.to_string_lossy().as_ref()).unwrap_err();
    assert_app_error(error, "invalid_repository", "当前目录不是 Git 仓库");
}

#[test]
fn disables_push_when_upstream_is_not_origin() {
    let repo = init_repo();
    let origin = init_bare_repo();
    let backup = init_bare_repo();
    commit_file(&repo.path, "file.txt", "hello\n", "initial");
    let branch = current_test_branch(&repo.path);

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(
        &repo.path,
        &["remote", "add", "backup", backup.path.to_str().unwrap()],
    );
    let backup_refspec = format!("HEAD:refs/heads/{branch}");
    run_git(&repo.path, &["push", "-u", "backup", &backup_refspec]);

    let status = branch_status_for_path(&repo.path).unwrap();
    let expected_upstream = format!("backup/{branch}");

    assert_eq!(status.upstream.as_deref(), Some(expected_upstream.as_str()));
    assert!(!status.push_available);
    assert_eq!(
        status.disabled_reason.as_deref(),
        Some("当前分支的上游不是 origin 远端，已禁用推送动作")
    );
}

#[test]
fn pushes_current_branch_to_origin_even_when_upstream_points_elsewhere() {
    let repo = init_repo();
    let origin = init_bare_repo();
    let backup = init_bare_repo();
    commit_file(&repo.path, "file.txt", "hello\n", "initial");
    let branch = current_test_branch(&repo.path);

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(
        &repo.path,
        &["remote", "add", "backup", backup.path.to_str().unwrap()],
    );
    let backup_refspec = format!("HEAD:refs/heads/{branch}");
    run_git(&repo.path, &["push", "-u", "backup", &backup_refspec]);
    let backup_initial_head = run_git(
        &backup.path,
        &["rev-parse", &format!("refs/heads/{branch}")],
    );

    let second_hash = commit_file(&repo.path, "file.txt", "hello\nworld\n", "second");

    push_current_branch_checked(repo.path.to_string_lossy().as_ref(), &branch).unwrap();

    let origin_head = run_git(
        &origin.path,
        &["rev-parse", &format!("refs/heads/{branch}")],
    );
    let backup_head = run_git(
        &backup.path,
        &["rev-parse", &format!("refs/heads/{branch}")],
    );

    assert_eq!(origin_head, second_hash);
    assert_eq!(backup_head, backup_initial_head);
}

#[test]
fn separates_total_history_from_safe_step_push_targets_after_merge() {
    let repo = init_repo();
    let origin = init_bare_repo();
    let base_hash = commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    run_git(&repo.path, &["checkout", "-b", "feature", &base_hash]);
    let feature_hash = commit_file(&repo.path, "feature.txt", "feature\n", "feature");

    run_git(&repo.path, &["checkout", &branch]);
    let main_hash = commit_file(&repo.path, "main.txt", "main\n", "main");
    run_git(
        &repo.path,
        &["merge", "--no-ff", "feature", "-m", "merge feature"],
    );
    let merge_hash = run_git(&repo.path, &["rev-parse", "HEAD"]);

    let status = branch_status_for_path(&repo.path).unwrap();
    assert_eq!(status.ahead_count, 3);
    assert_eq!(status.safe_ahead_count, 2);
    assert_eq!(status.behind_count, 0);

    let history = get_commit_history(repo.path.to_string_lossy().as_ref(), 0, 10, None).unwrap();
    let hashes: Vec<&str> = history
        .items
        .iter()
        .map(|item| item.hash.as_str())
        .collect();

    assert_eq!(history.unpushed_count, 3);
    assert_eq!(history.safe_unpushed_count, 2);
    assert!(hashes.contains(&merge_hash.as_str()));
    assert!(hashes.contains(&main_hash.as_str()));
    assert!(hashes.contains(&feature_hash.as_str()));
    assert!(hashes.contains(&base_hash.as_str()));

    let merge_item = history
        .items
        .iter()
        .find(|item| item.hash == merge_hash)
        .expect("merge commit in history");
    let main_item = history
        .items
        .iter()
        .find(|item| item.hash == main_hash)
        .expect("main commit in history");
    let feature_item = history
        .items
        .iter()
        .find(|item| item.hash == feature_hash)
        .expect("feature commit in history");
    let base_item = history
        .items
        .iter()
        .find(|item| item.hash == base_hash)
        .expect("base commit in history");

    assert!(!merge_item.is_pushed);
    assert!(merge_item.is_safe_push_target);
    assert_eq!(merge_item.push_blocked_reason, None);
    assert!(!main_item.is_pushed);
    assert!(main_item.is_safe_push_target);
    assert_eq!(main_item.push_blocked_reason, None);
    assert!(!feature_item.is_pushed);
    assert!(!feature_item.is_safe_push_target);
    assert_eq!(
        feature_item.push_blocked_reason.as_deref(),
        Some(UNSAFE_PUSH_TARGET_MESSAGE)
    );
    assert!(base_item.is_pushed);
}

#[test]
fn builds_safe_step_push_plan_for_linear_ahead_commits() {
    let repo = init_repo();
    let origin = init_bare_repo();
    commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    let first_hash = commit_file(&repo.path, "first.txt", "first\n", "first");
    let second_hash = commit_file(&repo.path, "second.txt", "second\n", "second");

    let status = branch_status_for_path(&repo.path).unwrap();
    assert_eq!(status.ahead_count, 2);
    assert_eq!(status.safe_ahead_count, 2);

    let plan = get_step_push_plan(repo.path.to_string_lossy().as_ref(), &second_hash).unwrap();
    let plan_hashes: Vec<&str> = plan.items.iter().map(|item| item.hash.as_str()).collect();

    assert!(plan.available);
    assert_eq!(plan_hashes, vec![first_hash.as_str(), second_hash.as_str()]);
}

#[test]
fn blocks_safe_step_push_plan_when_branch_has_diverged() {
    let repo = init_repo();
    let origin = init_bare_repo();
    commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    let peer = clone_repo(&origin.path, "peer");
    commit_file(&peer.path, "remote.txt", "remote\n", "remote");
    run_git(&peer.path, &["push", "origin", &format!("HEAD:{branch}")]);

    let local_hash = commit_file(&repo.path, "local.txt", "local\n", "local");
    refresh_remote_tracking(repo.path.to_string_lossy().as_ref()).unwrap();

    let status = branch_status_for_path(&repo.path).unwrap();
    assert_eq!(status.behind_count, 1);
    assert_eq!(status.safe_ahead_count, 0);

    let plan = get_step_push_plan(repo.path.to_string_lossy().as_ref(), &local_hash).unwrap();
    let reason = plan.blocked_reason.expect("blocked plan reason");

    assert!(!plan.available);
    assert_eq!(reason.code, "behind_remote");
}

#[test]
fn filters_commit_history_by_summary_and_body_case_insensitively() {
    let repo = init_repo();
    let first_hash = commit_file(&repo.path, "first.txt", "first\n", "Initial setup");
    let body_hash = commit_file_with_body(
        &repo.path,
        "body.txt",
        "body\n",
        "Refactor internals",
        "Includes a Release Note marker",
    );
    let title_hash = commit_file(&repo.path, "title.txt", "title\n", "Add Search Panel");

    let title_history = get_commit_history(
        repo.path.to_string_lossy().as_ref(),
        0,
        10,
        Some(CommitHistoryFilters {
            query: Some("search".to_string()),
            ..CommitHistoryFilters::default()
        }),
    )
    .unwrap();

    assert_eq!(title_history.items.len(), 1);
    assert_eq!(title_history.items[0].hash, title_hash);
    assert_eq!(title_history.next_skip, 1);
    assert_eq!(title_history.total_count, 1);
    assert!(!title_history.has_more);

    let body_history = get_commit_history(
        repo.path.to_string_lossy().as_ref(),
        0,
        10,
        Some(CommitHistoryFilters {
            query: Some("release note".to_string()),
            ..CommitHistoryFilters::default()
        }),
    )
    .unwrap();

    assert_eq!(body_history.items.len(), 1);
    assert_eq!(body_history.items[0].hash, body_hash);
    assert_eq!(body_history.total_count, 1);
    assert!(body_history
        .items
        .iter()
        .all(|item| item.hash != first_hash));
}

#[test]
fn filters_commit_history_by_message_separately_from_global_query() {
    let repo = init_repo();
    run_git(&repo.path, &["config", "user.name", "Alice Example"]);
    run_git(&repo.path, &["config", "user.email", "alice@example.com"]);
    let alice_hash = commit_file(&repo.path, "alice.txt", "alice\n", "plain work");
    run_git(&repo.path, &["config", "user.name", "TickGit Tests"]);
    run_git(
        &repo.path,
        &["config", "user.email", "tickgit-tests@example.com"],
    );
    let body_hash = commit_file_with_body(
        &repo.path,
        "body.txt",
        "body\n",
        "Refactor internals",
        "Includes a Release Note marker",
    );

    let global_history = get_commit_history(
        repo.path.to_string_lossy().as_ref(),
        0,
        10,
        Some(CommitHistoryFilters {
            query: Some("alice".to_string()),
            ..CommitHistoryFilters::default()
        }),
    )
    .unwrap();
    let message_history = get_commit_history(
        repo.path.to_string_lossy().as_ref(),
        0,
        10,
        Some(CommitHistoryFilters {
            message: Some("release note".to_string()),
            ..CommitHistoryFilters::default()
        }),
    )
    .unwrap();
    let and_history = get_commit_history(
        repo.path.to_string_lossy().as_ref(),
        0,
        10,
        Some(CommitHistoryFilters {
            query: Some("alice".to_string()),
            message: Some("release note".to_string()),
            ..CommitHistoryFilters::default()
        }),
    )
    .unwrap();

    assert_eq!(global_history.items.len(), 1);
    assert_eq!(global_history.items[0].hash, alice_hash);
    assert_eq!(global_history.total_count, 1);
    assert_eq!(message_history.items.len(), 1);
    assert_eq!(message_history.items[0].hash, body_hash);
    assert_eq!(message_history.total_count, 1);
    assert_eq!(and_history.items.len(), 0);
    assert_eq!(and_history.total_count, 0);
}

#[test]
fn filters_commit_history_by_author_name_and_email_case_insensitively() {
    let repo = init_repo();
    commit_file(&repo.path, "first.txt", "first\n", "first");
    run_git(&repo.path, &["config", "user.name", "Alice Example"]);
    run_git(&repo.path, &["config", "user.email", "alice@example.com"]);
    let alice_hash = commit_file(&repo.path, "alice.txt", "alice\n", "alice commit");
    run_git(&repo.path, &["config", "user.name", "Bob Example"]);
    run_git(&repo.path, &["config", "user.email", "bob@example.com"]);
    let bob_hash = commit_file(&repo.path, "bob.txt", "bob\n", "bob commit");

    let name_history = get_commit_history(
        repo.path.to_string_lossy().as_ref(),
        0,
        10,
        Some(CommitHistoryFilters {
            author: Some("alice".to_string()),
            ..CommitHistoryFilters::default()
        }),
    )
    .unwrap();

    assert_eq!(name_history.items.len(), 1);
    assert_eq!(name_history.items[0].hash, alice_hash);
    assert_eq!(name_history.total_count, 1);

    let email_history = get_commit_history(
        repo.path.to_string_lossy().as_ref(),
        0,
        10,
        Some(CommitHistoryFilters {
            author: Some("BOB@EXAMPLE".to_string()),
            ..CommitHistoryFilters::default()
        }),
    )
    .unwrap();

    assert_eq!(email_history.items.len(), 1);
    assert_eq!(email_history.items[0].hash, bob_hash);
    assert_eq!(email_history.total_count, 1);
}

#[test]
fn paginates_filtered_commit_history_after_metadata_filters() {
    let repo = init_repo();
    let first_hash = commit_file(&repo.path, "one.txt", "one\n", "ticket one");
    let second_hash = commit_file(&repo.path, "two.txt", "two\n", "ticket two");
    let third_hash = commit_file(&repo.path, "three.txt", "three\n", "ticket three");
    commit_file(&repo.path, "other.txt", "other\n", "other");

    let first_page = get_commit_history(
        repo.path.to_string_lossy().as_ref(),
        0,
        2,
        Some(CommitHistoryFilters {
            query: Some("ticket".to_string()),
            ..CommitHistoryFilters::default()
        }),
    )
    .unwrap();
    let second_page = get_commit_history(
        repo.path.to_string_lossy().as_ref(),
        first_page.next_skip,
        2,
        Some(CommitHistoryFilters {
            query: Some("ticket".to_string()),
            ..CommitHistoryFilters::default()
        }),
    )
    .unwrap();

    assert_eq!(
        first_page
            .items
            .iter()
            .map(|item| item.hash.as_str())
            .collect::<Vec<_>>(),
        vec![third_hash.as_str(), second_hash.as_str()]
    );
    assert!(first_page.has_more);
    assert_eq!(first_page.next_skip, 2);
    assert_eq!(first_page.total_count, 3);
    assert_eq!(second_page.items.len(), 1);
    assert_eq!(second_page.items[0].hash, first_hash);
    assert!(!second_page.has_more);
    assert_eq!(second_page.next_skip, 3);
    assert_eq!(second_page.total_count, 3);
}

#[test]
fn filters_commit_history_by_file_path_fragments() {
    let repo = init_repo();
    let docs_hash = commit_file(&repo.path, "docs/plan.md", "plan\n", "docs plan");
    commit_file(&repo.path, "src/main.rs", "main\n", "main code");
    commit_file(&repo.path, "README.md", "readme\n", "readme");

    let history = get_commit_history(
        repo.path.to_string_lossy().as_ref(),
        0,
        10,
        Some(CommitHistoryFilters {
            file_path: Some("DOCS/plan".to_string()),
            ..CommitHistoryFilters::default()
        }),
    )
    .unwrap();

    assert_eq!(history.items.len(), 1);
    assert_eq!(history.items[0].hash, docs_hash);
    assert_eq!(history.next_skip, 1);
    assert!(!history.has_more);
}

#[test]
fn filters_commit_history_by_renamed_previous_and_new_paths() {
    let repo = init_repo();
    commit_file(&repo.path, "src/old-name.txt", "before\n", "create old");
    run_git(&repo.path, &["mv", "src/old-name.txt", "src/new-name.txt"]);
    run_git(
        &repo.path,
        &["commit", "--no-gpg-sign", "-m", "rename file"],
    );
    let rename_hash = run_git(&repo.path, &["rev-parse", "HEAD"]);

    let old_path_history = get_commit_history(
        repo.path.to_string_lossy().as_ref(),
        0,
        10,
        Some(CommitHistoryFilters {
            file_path: Some("old-name".to_string()),
            ..CommitHistoryFilters::default()
        }),
    )
    .unwrap();
    let new_path_history = get_commit_history(
        repo.path.to_string_lossy().as_ref(),
        0,
        10,
        Some(CommitHistoryFilters {
            file_path: Some("new-name".to_string()),
            ..CommitHistoryFilters::default()
        }),
    )
    .unwrap();

    assert!(old_path_history
        .items
        .iter()
        .any(|item| item.hash == rename_hash));
    assert_eq!(new_path_history.items[0].hash, rename_hash);
}

#[test]
fn paginates_commit_history_after_file_path_filters() {
    let repo = init_repo();
    let first_hash = commit_file(&repo.path, "tickets/one.txt", "one\n", "ticket one");
    let second_hash = commit_file(&repo.path, "tickets/two.txt", "two\n", "ticket two");
    let third_hash = commit_file(&repo.path, "tickets/three.txt", "three\n", "ticket three");
    commit_file(&repo.path, "notes/other.txt", "other\n", "other");

    let first_page = get_commit_history(
        repo.path.to_string_lossy().as_ref(),
        0,
        2,
        Some(CommitHistoryFilters {
            file_path: Some("tickets/".to_string()),
            ..CommitHistoryFilters::default()
        }),
    )
    .unwrap();
    let second_page = get_commit_history(
        repo.path.to_string_lossy().as_ref(),
        first_page.next_skip,
        2,
        Some(CommitHistoryFilters {
            file_path: Some("tickets/".to_string()),
            ..CommitHistoryFilters::default()
        }),
    )
    .unwrap();

    assert_eq!(
        first_page
            .items
            .iter()
            .map(|item| item.hash.as_str())
            .collect::<Vec<_>>(),
        vec![third_hash.as_str(), second_hash.as_str()]
    );
    assert!(first_page.has_more);
    assert_eq!(second_page.items.len(), 1);
    assert_eq!(second_page.items[0].hash, first_hash);
    assert!(!second_page.has_more);
}

#[test]
fn keeps_unpushed_commit_flags_when_file_path_filters_are_active() {
    let repo = init_repo();
    let origin = init_bare_repo();
    commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    let local_hash = commit_file(&repo.path, "local/path.txt", "local\n", "local path");

    let history = get_commit_history(
        repo.path.to_string_lossy().as_ref(),
        0,
        10,
        Some(CommitHistoryFilters {
            file_path: Some("local/path".to_string()),
            ..CommitHistoryFilters::default()
        }),
    )
    .unwrap();

    assert_eq!(history.items.len(), 1);
    assert_eq!(history.items[0].hash, local_hash);
    assert!(!history.items[0].is_pushed);
    assert!(history.items[0].is_safe_push_target);
    assert_eq!(history.unpushed_count, 1);
    assert_eq!(history.safe_unpushed_count, 1);
}

#[test]
fn rejects_push_to_commit_for_unsafe_merge_side_branch_commit() {
    let repo = init_repo();
    let origin = init_bare_repo();
    let base_hash = commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    run_git(&repo.path, &["checkout", "-b", "feature", &base_hash]);
    let feature_hash = commit_file(&repo.path, "feature.txt", "feature\n", "feature");

    run_git(&repo.path, &["checkout", &branch]);
    commit_file(&repo.path, "main.txt", "main\n", "main");
    run_git(
        &repo.path,
        &["merge", "--no-ff", "feature", "-m", "merge feature"],
    );

    let error =
        push_to_commit(repo.path.to_string_lossy().as_ref(), &branch, &feature_hash).unwrap_err();
    assert_eq!(error.code, "unsafe_push_target");
    assert_eq!(error.message, UNSAFE_PUSH_TARGET_MESSAGE);
}

#[test]
fn validates_requested_branch_must_match_current_branch() {
    let repo = init_repo();
    commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);

    let current = validate_current_branch(repo.path.to_string_lossy().as_ref(), &branch).unwrap();
    assert_eq!(current, branch);

    let mismatch =
        validate_current_branch(repo.path.to_string_lossy().as_ref(), "not-current").unwrap_err();
    assert_app_error(mismatch, "branch_mismatch", BRANCH_MISMATCH_MESSAGE);

    let empty = validate_current_branch(repo.path.to_string_lossy().as_ref(), "   ").unwrap_err();
    assert_app_error(empty, "invalid_branch", "目标分支不能为空");
}

#[test]
fn rejects_push_to_commit_when_requested_branch_differs_from_current() {
    let repo = init_repo();
    let origin = init_bare_repo();
    let base_hash = commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    let safe_hash = commit_file(&repo.path, "safe.txt", "safe\n", "safe");
    let error = push_to_commit(
        repo.path.to_string_lossy().as_ref(),
        "not-current",
        &safe_hash,
    )
    .unwrap_err();

    assert_app_error(error, "branch_mismatch", BRANCH_MISMATCH_MESSAGE);

    let origin_head = run_git(
        &origin.path,
        &["rev-parse", &format!("refs/heads/{branch}")],
    );
    assert_eq!(origin_head, base_hash);
}

#[test]
fn allows_push_to_commit_for_safe_first_parent_commit() {
    let repo = init_repo();
    let origin = init_bare_repo();
    commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    let safe_hash = commit_file(&repo.path, "safe.txt", "safe\n", "safe");

    push_to_commit(repo.path.to_string_lossy().as_ref(), &branch, &safe_hash).unwrap();

    let origin_head = run_git(
        &origin.path,
        &["rev-parse", &format!("refs/heads/{branch}")],
    );
    assert_eq!(origin_head, safe_hash);
}

#[test]
fn accepts_contiguous_step_push_hashes() {
    let repo = init_repo();
    let origin = init_bare_repo();
    commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    let first_hash = commit_file(&repo.path, "first.txt", "first\n", "first");
    let second_hash = commit_file(&repo.path, "second.txt", "second\n", "second");
    commit_file(&repo.path, "third.txt", "third\n", "third");

    validate_step_push_hashes(
        repo.path.to_string_lossy().as_ref(),
        &[first_hash, second_hash],
    )
    .unwrap();
}

#[test]
fn builds_step_push_plan_for_safe_first_parent_target() {
    let repo = init_repo();
    let origin = init_bare_repo();
    commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    let first_hash = commit_file(&repo.path, "first.txt", "first\n", "first");
    let second_hash = commit_file(&repo.path, "second.txt", "second\n", "second");
    commit_file(&repo.path, "third.txt", "third\n", "third");

    let plan = get_step_push_plan(repo.path.to_string_lossy().as_ref(), &second_hash).unwrap();

    assert!(plan.available);
    assert_eq!(plan.branch, branch);
    assert_eq!(plan.target_hash, second_hash);
    assert!(plan.blocked_reason.is_none());
    assert_eq!(
        plan.items
            .iter()
            .map(|item| item.hash.as_str())
            .collect::<Vec<_>>(),
        vec![first_hash.as_str(), second_hash.as_str()]
    );
    assert_eq!(plan.items[0].summary, "first");
    assert!(!plan.items[0].short_hash.is_empty());
}

#[test]
fn blocks_step_push_plan_for_merge_side_branch_commit() {
    let repo = init_repo();
    let origin = init_bare_repo();
    let base_hash = commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    run_git(&repo.path, &["checkout", "-b", "feature", &base_hash]);
    let feature_hash = commit_file(&repo.path, "feature.txt", "feature\n", "feature");

    run_git(&repo.path, &["checkout", &branch]);
    commit_file(&repo.path, "main.txt", "main\n", "main");
    run_git(
        &repo.path,
        &["merge", "--no-ff", "feature", "-m", "merge feature"],
    );

    let plan = get_step_push_plan(repo.path.to_string_lossy().as_ref(), &feature_hash).unwrap();
    let reason = plan.blocked_reason.expect("blocked reason");

    assert!(!plan.available);
    assert!(plan.items.is_empty());
    assert_eq!(reason.code, "unsafe_push_target");
    assert_eq!(reason.message, UNSAFE_PUSH_TARGET_MESSAGE);
}

#[test]
fn blocks_step_push_plan_when_remote_advanced_without_fetch() {
    let repo = init_repo();
    let origin = init_bare_repo();
    commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    let peer = clone_repo(&origin.path, "peer");
    commit_file(&peer.path, "remote.txt", "remote\n", "remote");
    run_git(&peer.path, &["push", "origin", &format!("HEAD:{branch}")]);

    let local_hash = commit_file(&repo.path, "local.txt", "local\n", "local");

    let plan = get_step_push_plan(repo.path.to_string_lossy().as_ref(), &local_hash).unwrap();
    let reason = plan.blocked_reason.expect("blocked reason");

    assert!(!plan.available);
    assert!(plan.items.is_empty());
    assert_eq!(reason.code, "behind_remote");
    assert_eq!(reason.message, BRANCH_BEHIND_REMOTE_MESSAGE);
}

#[test]
fn blocks_step_push_plan_when_upstream_is_missing() {
    let repo = init_repo();
    let origin = init_bare_repo();
    let target_hash = commit_file(&repo.path, "file.txt", "hello\n", "initial");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );

    let plan = get_step_push_plan(repo.path.to_string_lossy().as_ref(), &target_hash).unwrap();
    let reason = plan.blocked_reason.expect("blocked reason");

    assert!(!plan.available);
    assert!(plan.items.is_empty());
    assert_eq!(reason.code, "missing_upstream");
    assert_eq!(reason.message, "当前分支没有上游跟踪分支，已禁用推送动作");
}

#[test]
fn blocks_step_push_plan_when_upstream_is_not_origin() {
    let repo = init_repo();
    let origin = init_bare_repo();
    let backup = init_bare_repo();
    commit_file(&repo.path, "file.txt", "hello\n", "initial");
    let branch = current_test_branch(&repo.path);

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(
        &repo.path,
        &["remote", "add", "backup", backup.path.to_str().unwrap()],
    );
    let backup_refspec = format!("HEAD:refs/heads/{branch}");
    run_git(&repo.path, &["push", "-u", "backup", &backup_refspec]);

    let target_hash = commit_file(&repo.path, "local.txt", "local\n", "local");
    let plan = get_step_push_plan(repo.path.to_string_lossy().as_ref(), &target_hash).unwrap();
    let reason = plan.blocked_reason.expect("blocked reason");

    assert!(!plan.available);
    assert!(plan.items.is_empty());
    assert_eq!(reason.code, "non_origin_upstream");
    assert_eq!(
        reason.message,
        "当前分支的上游不是 origin 远端，已禁用推送动作"
    );
}

#[test]
fn rejects_non_contiguous_step_push_hashes() {
    let repo = init_repo();
    let origin = init_bare_repo();
    commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    let first_hash = commit_file(&repo.path, "first.txt", "first\n", "first");
    commit_file(&repo.path, "second.txt", "second\n", "second");
    let third_hash = commit_file(&repo.path, "third.txt", "third\n", "third");

    let error = validate_step_push_hashes(
        repo.path.to_string_lossy().as_ref(),
        &[first_hash, third_hash],
    )
    .unwrap_err();

    assert_eq!(error.code, "unsafe_push_target");
    assert_eq!(error.message, UNSAFE_PUSH_TARGET_MESSAGE);
}

#[test]
fn rejects_push_to_commit_when_remote_advanced_without_fetch() {
    let repo = init_repo();
    let origin = init_bare_repo();
    commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    let peer = clone_repo(&origin.path, "peer");
    commit_file(&peer.path, "remote.txt", "remote\n", "remote");
    run_git(&peer.path, &["push", "origin", &format!("HEAD:{branch}")]);

    let local_hash = commit_file(&repo.path, "local.txt", "local\n", "local");

    let error =
        push_to_commit(repo.path.to_string_lossy().as_ref(), &branch, &local_hash).unwrap_err();

    assert_eq!(error.code, "push_unavailable");
    assert_eq!(error.message, BRANCH_BEHIND_REMOTE_MESSAGE);
}

#[test]
fn rejects_step_push_validation_when_remote_advanced_without_fetch() {
    let repo = init_repo();
    let origin = init_bare_repo();
    commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    let peer = clone_repo(&origin.path, "peer");
    commit_file(&peer.path, "remote.txt", "remote\n", "remote");
    run_git(&peer.path, &["push", "origin", &format!("HEAD:{branch}")]);

    let local_hash = commit_file(&repo.path, "local.txt", "local\n", "local");

    let error =
        validate_step_push_hashes(repo.path.to_string_lossy().as_ref(), &[local_hash]).unwrap_err();

    assert_eq!(error.code, "push_unavailable");
    assert_eq!(error.message, BRANCH_BEHIND_REMOTE_MESSAGE);
}

#[test]
fn refreshes_remote_tracking_and_marks_branch_behind_remote() {
    let repo = init_repo();
    let origin = init_bare_repo();
    commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    let peer = clone_repo(&origin.path, "peer");
    commit_file(&peer.path, "remote.txt", "remote\n", "remote");
    run_git(&peer.path, &["push", "origin", &format!("HEAD:{branch}")]);

    commit_file(&repo.path, "local.txt", "local\n", "local");

    refresh_remote_tracking(repo.path.to_string_lossy().as_ref()).unwrap();

    let status = branch_status_for_path(&repo.path).unwrap();
    assert_eq!(status.behind_count, 1);
    assert!(!status.push_available);
    assert_eq!(
        status.disabled_reason.as_deref(),
        Some(BRANCH_BEHIND_REMOTE_MESSAGE)
    );
}

#[test]
fn keeps_unpushed_commits_visible_when_branch_is_behind_remote() {
    let repo = init_repo();
    let origin = init_bare_repo();
    commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    let peer = clone_repo(&origin.path, "peer");
    commit_file(&peer.path, "remote.txt", "remote\n", "remote");
    run_git(&peer.path, &["push", "origin", &format!("HEAD:{branch}")]);

    let local_hash = commit_file(&repo.path, "local.txt", "local\n", "local");

    refresh_remote_tracking(repo.path.to_string_lossy().as_ref()).unwrap();

    let history = get_commit_history(repo.path.to_string_lossy().as_ref(), 0, 10, None).unwrap();
    let status = branch_status_for_path(&repo.path).unwrap();
    let local_item = history
        .items
        .iter()
        .find(|item| item.hash == local_hash)
        .expect("local commit in history");

    assert_eq!(status.behind_count, 1);
    assert!(!status.push_available);
    assert_eq!(status.safe_ahead_count, 0);
    assert!(!local_item.is_pushed);
    assert!(!local_item.is_safe_push_target);
    assert_eq!(
        local_item.push_blocked_reason.as_deref(),
        Some(BRANCH_BEHIND_REMOTE_MESSAGE)
    );
    assert_eq!(history.unpushed_count, 1);
    assert_eq!(history.safe_unpushed_count, 0);
}

#[test]
fn limits_safe_step_push_targets_to_fast_forwardable_suffix_when_remote_is_merged_side_parent() {
    let repo = init_repo();
    let origin = init_bare_repo();
    commit_file(&repo.path, "base.txt", "base\n", "base");
    let branch = current_test_branch(&repo.path);
    let refspec = format!("HEAD:refs/heads/{branch}");

    run_git(
        &repo.path,
        &["remote", "add", "origin", origin.path.to_str().unwrap()],
    );
    run_git(&repo.path, &["push", "-u", "origin", &refspec]);

    let peer = clone_repo(&origin.path, "peer");
    commit_file(&peer.path, "remote.txt", "remote\n", "remote");
    run_git(&peer.path, &["push", "origin", &format!("HEAD:{branch}")]);

    let local_main_hash = commit_file(&repo.path, "main.txt", "main\n", "main");
    refresh_remote_tracking(repo.path.to_string_lossy().as_ref()).unwrap();
    run_git(
        &repo.path,
        &[
            "merge",
            "--no-ff",
            &format!("origin/{branch}"),
            "-m",
            "merge remote",
        ],
    );
    let merge_hash = run_git(&repo.path, &["rev-parse", "HEAD"]);

    let status = branch_status_for_path(&repo.path).unwrap();
    assert_eq!(status.behind_count, 0);
    assert_eq!(status.ahead_count, 2);
    assert_eq!(status.safe_ahead_count, 1);

    let history = get_commit_history(repo.path.to_string_lossy().as_ref(), 0, 10, None).unwrap();
    let merge_item = history
        .items
        .iter()
        .find(|item| item.hash == merge_hash)
        .expect("merge commit in history");
    let main_item = history
        .items
        .iter()
        .find(|item| item.hash == local_main_hash)
        .expect("main commit in history");

    assert!(merge_item.is_safe_push_target);
    assert!(!main_item.is_safe_push_target);
    assert_eq!(history.safe_unpushed_count, 1);

    validate_step_push_hashes(
        repo.path.to_string_lossy().as_ref(),
        std::slice::from_ref(&merge_hash),
    )
    .unwrap();

    let error = validate_step_push_hashes(
        repo.path.to_string_lossy().as_ref(),
        &[local_main_hash, merge_hash],
    )
    .unwrap_err();
    assert_eq!(error.code, "unsafe_push_target");
    assert_eq!(error.message, UNSAFE_PUSH_TARGET_MESSAGE);
}

#[test]
fn gets_diff_for_initial_commit() {
    let repo = init_repo();
    let initial_hash = commit_file(&repo.path, "file.txt", "hello\n", "initial");

    let diff = get_commit_file_diff(
        repo.path.to_string_lossy().as_ref(),
        &initial_hash,
        "file.txt",
        None,
        false,
    )
    .unwrap();

    assert!(diff.text.contains("diff --git"));
    assert!(diff.text.contains("+hello"));
}

#[test]
fn gets_diff_for_non_initial_commit() {
    let repo = init_repo();
    commit_file(&repo.path, "file.txt", "hello\n", "initial");
    let second_hash = commit_file(&repo.path, "file.txt", "hello\nworld\n", "second");

    let diff = get_commit_file_diff(
        repo.path.to_string_lossy().as_ref(),
        &second_hash,
        "file.txt",
        None,
        false,
    )
    .unwrap();

    assert!(diff.text.contains("@@"));
    assert!(diff.text.contains("+world"));
}

#[test]
fn hides_whitespace_only_diff_for_non_initial_commit() {
    let repo = init_repo();
    commit_file(&repo.path, "file.txt", "hello\n", "initial");
    let second_hash = commit_file(&repo.path, "file.txt", "hello \n", "whitespace");

    let normal_diff = get_commit_file_diff(
        repo.path.to_string_lossy().as_ref(),
        &second_hash,
        "file.txt",
        None,
        false,
    )
    .unwrap();
    let hidden_diff = get_commit_file_diff(
        repo.path.to_string_lossy().as_ref(),
        &second_hash,
        "file.txt",
        None,
        true,
    )
    .unwrap();

    assert!(normal_diff.text.contains("@@"));
    assert!(hidden_diff.text.trim().is_empty());
    assert!(!hidden_diff.is_too_large);
    assert!(!hidden_diff.is_binary);
}

#[test]
fn gets_diff_for_initial_commit_when_hiding_whitespace() {
    let repo = init_repo();
    let initial_hash = commit_file(&repo.path, "file.txt", "   \n", "initial");

    let hidden_diff = get_commit_file_diff(
        repo.path.to_string_lossy().as_ref(),
        &initial_hash,
        "file.txt",
        None,
        true,
    )
    .unwrap();

    assert!(hidden_diff.text.contains("diff --git"));
    assert!(hidden_diff.text.contains("file.txt"));
}

#[test]
fn gets_rename_diff_when_previous_path_is_available() {
    let repo = init_repo();
    commit_file(&repo.path, "old.txt", "hello\n", "initial");
    run_git(&repo.path, &["mv", "old.txt", "new.txt"]);
    run_git(&repo.path, &["commit", "--no-gpg-sign", "-m", "rename"]);
    let rename_hash = run_git(&repo.path, &["rev-parse", "HEAD"]);

    let diff = get_commit_file_diff(
        repo.path.to_string_lossy().as_ref(),
        &rename_hash,
        "new.txt",
        Some("old.txt"),
        false,
    )
    .unwrap();

    assert!(diff.text.contains("rename from old.txt"));
    assert!(diff.text.contains("rename to new.txt"));
}

#[test]
fn marks_binary_diff_without_text_patch() {
    let repo = init_repo();
    fs::write(repo.path.join("data.bin"), b"before\0after").expect("write binary file");
    run_git(&repo.path, &["add", "data.bin"]);
    run_git(&repo.path, &["commit", "--no-gpg-sign", "-m", "binary"]);
    let hash = run_git(&repo.path, &["rev-parse", "HEAD"]);

    let diff = get_commit_file_diff(
        repo.path.to_string_lossy().as_ref(),
        &hash,
        "data.bin",
        None,
        false,
    )
    .unwrap();

    assert!(diff.is_binary);
    assert!(!diff.is_image);
    assert!(diff.text.is_empty());
}

#[test]
fn marks_image_diff_by_extension() {
    let repo = init_repo();
    fs::write(repo.path.join("image.png"), b"\x89PNG\r\n\x1a\n\0data").expect("write image file");
    run_git(&repo.path, &["add", "image.png"]);
    run_git(&repo.path, &["commit", "--no-gpg-sign", "-m", "image"]);
    let hash = run_git(&repo.path, &["rev-parse", "HEAD"]);

    let diff = get_commit_file_diff(
        repo.path.to_string_lossy().as_ref(),
        &hash,
        "image.png",
        None,
        false,
    )
    .unwrap();

    assert!(diff.is_image);
    assert!(diff.text.is_empty());
}

#[test]
fn returns_commit_image_data_urls_for_added_modified_and_deleted_images() {
    let repo = init_repo();
    write_file(&repo.path, "image.svg", &svg_content("one"));
    run_git(&repo.path, &["add", "image.svg"]);
    run_git(&repo.path, &["commit", "--no-gpg-sign", "-m", "add image"]);
    let added_hash = run_git(&repo.path, &["rev-parse", "HEAD"]);

    write_file(&repo.path, "image.svg", &svg_content("two"));
    run_git(&repo.path, &["add", "image.svg"]);
    run_git(
        &repo.path,
        &["commit", "--no-gpg-sign", "-m", "modify image"],
    );
    let modified_hash = run_git(&repo.path, &["rev-parse", "HEAD"]);

    fs::remove_file(repo.path.join("image.svg")).expect("delete image");
    run_git(&repo.path, &["add", "image.svg"]);
    run_git(
        &repo.path,
        &["commit", "--no-gpg-sign", "-m", "delete image"],
    );
    let deleted_hash = run_git(&repo.path, &["rev-parse", "HEAD"]);

    let added_diff = get_commit_file_diff(
        repo.path.to_string_lossy().as_ref(),
        &added_hash,
        "image.svg",
        None,
        false,
    )
    .unwrap();
    let modified_diff = get_commit_file_diff(
        repo.path.to_string_lossy().as_ref(),
        &modified_hash,
        "image.svg",
        None,
        false,
    )
    .unwrap();
    let deleted_diff = get_commit_file_diff(
        repo.path.to_string_lossy().as_ref(),
        &deleted_hash,
        "image.svg",
        None,
        false,
    )
    .unwrap();

    assert!(added_diff.is_image);
    assert!(added_diff.old_image_data_url.is_none());
    assert_svg_data_url(added_diff.new_image_data_url.as_ref());
    assert!(modified_diff.is_image);
    assert_svg_data_url(modified_diff.old_image_data_url.as_ref());
    assert_svg_data_url(modified_diff.new_image_data_url.as_ref());
    assert!(deleted_diff.is_image);
    assert_svg_data_url(deleted_diff.old_image_data_url.as_ref());
    assert!(deleted_diff.new_image_data_url.is_none());
}

#[test]
fn skips_text_for_too_large_diff() {
    let repo = init_repo();
    let content = (0..5001)
        .map(|index| format!("line-{index}\n"))
        .collect::<String>();
    write_file(&repo.path, "large.txt", &content);
    run_git(&repo.path, &["add", "large.txt"]);
    run_git(&repo.path, &["commit", "--no-gpg-sign", "-m", "large"]);
    let hash = run_git(&repo.path, &["rev-parse", "HEAD"]);

    let diff = get_commit_file_diff(
        repo.path.to_string_lossy().as_ref(),
        &hash,
        "large.txt",
        None,
        false,
    )
    .unwrap();

    assert!(diff.is_too_large);
    assert!(diff.truncated);
    assert_eq!(diff.line_count, 5001);
    assert!(diff.text.is_empty());
}

#[test]
fn gets_commit_meta() {
    let repo = init_repo();
    write_file(&repo.path, "file.txt", "before\n");
    run_git(&repo.path, &["add", "file.txt"]);
    run_git(
        &repo.path,
        &[
            "commit",
            "--no-gpg-sign",
            "-m",
            "summary",
            "-m",
            "details line",
        ],
    );
    write_file(&repo.path, "file.txt", "before\nafter\n");
    run_git(&repo.path, &["add", "file.txt"]);
    run_git(
        &repo.path,
        &[
            "commit",
            "--no-gpg-sign",
            "-m",
            "follow up",
            "-m",
            "more context",
        ],
    );

    let hash = run_git(&repo.path, &["rev-parse", "HEAD"]);
    let meta = get_commit_meta(repo.path.to_string_lossy().as_ref(), &hash).unwrap();

    assert_eq!(meta.body, "more context");
    assert_eq!(meta.additions, 1);
    assert_eq!(meta.deletions, 0);
}

#[test]
fn lists_local_branches() {
    let repo = init_repo();
    commit_file(&repo.path, "file.txt", "hello\n", "initial");
    let current_branch = current_test_branch(&repo.path);
    run_git(&repo.path, &["branch", "feature"]);

    let branches = list_local_branches(repo.path.to_string_lossy().as_ref()).unwrap();

    assert_eq!(branches, vec!["feature".to_string(), current_branch]);
}

#[test]
fn checks_out_selected_local_branch() {
    let repo = init_repo();
    commit_file(&repo.path, "file.txt", "hello\n", "initial");
    run_git(&repo.path, &["branch", "feature"]);

    checkout_branch(repo.path.to_string_lossy().as_ref(), "feature").unwrap();

    let status = branch_status_for_path(&repo.path).unwrap();
    assert_eq!(status.branch, "feature");
}

#[test]
fn rejects_empty_checkout_branch() {
    let repo = init_repo();
    commit_file(&repo.path, "file.txt", "hello\n", "initial");
    let before = branch_status_for_path(&repo.path).unwrap().branch;

    let error = checkout_branch(repo.path.to_string_lossy().as_ref(), "   ").unwrap_err();

    assert_app_error(error, "invalid_branch", "目标分支不能为空");
    let after = branch_status_for_path(&repo.path).unwrap().branch;
    assert_eq!(after, before);
}
