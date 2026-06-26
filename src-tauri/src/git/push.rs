use std::{collections::HashSet, path::Path, process::Command, time::Instant};

use crate::{
    error::{AppError, AppResult},
    models::{StepPushPlan, StepPushPlanBlockedReason, StepPushPlanItem},
};

use super::{
    command::{git_run, git_text, git_trimmed},
    repository::{
        branch_status_for_path, current_branch_matching, resolve_repository_path,
        sync_origin_tracking,
    },
    BRANCH_BEHIND_REMOTE_MESSAGE, REMOTE_NAME, UNSAFE_PUSH_TARGET_MESSAGE,
};

fn step_push_plan_perf_enabled() -> bool {
    std::env::var("TICKGIT_PERF").is_ok_and(|value| value == "1")
}

fn log_step_push_plan_stage(
    stage: &str,
    started_at: Instant,
    target_hash: &str,
    branch: Option<&str>,
    upstream: Option<&str>,
    result: &str,
) {
    if !step_push_plan_perf_enabled() {
        return;
    }

    eprintln!(
        "[tickgit:perf] step_push_plan stage={stage} target_hash={target_hash} branch={} upstream={} result={result} elapsed_ms={:.1}",
        branch.unwrap_or("-"),
        upstream.unwrap_or("-"),
        started_at.elapsed().as_secs_f64() * 1000.0,
    );
}

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

fn safe_unpushed_hashes_in_push_order(repo_path: &Path, upstream: &str) -> AppResult<Vec<String>> {
    if !is_ancestor(repo_path, upstream, "HEAD")? {
        return Ok(Vec::new());
    }

    let range = format!("{upstream}..HEAD");
    let output = git_trimmed(
        repo_path,
        &[
            "rev-list",
            "--first-parent",
            "--ancestry-path",
            "--reverse",
            &range,
        ],
    )?;

    Ok(output
        .lines()
        .map(str::trim)
        .filter(|hash| !hash.is_empty())
        .map(ToOwned::to_owned)
        .collect())
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

fn blocked_step_push_plan(
    branch: String,
    target_hash: String,
    code: impl Into<String>,
    message: impl Into<String>,
) -> StepPushPlan {
    StepPushPlan {
        branch,
        target_hash,
        available: false,
        items: Vec::new(),
        blocked_reason: Some(StepPushPlanBlockedReason {
            code: code.into(),
            message: message.into(),
        }),
    }
}

fn step_push_branch_blocked_reason(
    branch_status: &crate::models::BranchStatus,
) -> Option<StepPushPlanBlockedReason> {
    let disabled_message = || {
        branch_status
            .disabled_reason
            .clone()
            .unwrap_or_else(|| "当前分支当前不可推送".to_string())
    };

    if branch_status.detached {
        Some(StepPushPlanBlockedReason {
            code: "detached_head".to_string(),
            message: disabled_message(),
        })
    } else if branch_status.upstream.is_none() {
        let message = disabled_message();
        let code = if message.contains("未配置 origin") {
            "missing_origin"
        } else {
            "missing_upstream"
        };

        Some(StepPushPlanBlockedReason {
            code: code.to_string(),
            message,
        })
    } else if !branch_status
        .upstream
        .as_deref()
        .is_some_and(|upstream| upstream.starts_with("origin/"))
    {
        Some(StepPushPlanBlockedReason {
            code: "non_origin_upstream".to_string(),
            message: disabled_message(),
        })
    } else if branch_status.behind_count > 0 {
        Some(StepPushPlanBlockedReason {
            code: "behind_remote".to_string(),
            message: BRANCH_BEHIND_REMOTE_MESSAGE.to_string(),
        })
    } else if !branch_status.push_available {
        Some(StepPushPlanBlockedReason {
            code: "push_unavailable".to_string(),
            message: disabled_message(),
        })
    } else {
        None
    }
}

fn parse_step_push_plan_item(record: &str) -> Option<StepPushPlanItem> {
    let mut fields = record.splitn(3, '\u{1f}');
    let hash = fields.next()?.trim();

    if hash.is_empty() {
        return None;
    }

    Some(StepPushPlanItem {
        hash: hash.to_string(),
        short_hash: fields.next().unwrap_or("").trim().to_string(),
        summary: fields.next().unwrap_or("").trim().to_string(),
    })
}

fn step_push_plan_items(repo_path: &Path, hashes: &[String]) -> AppResult<Vec<StepPushPlanItem>> {
    if hashes.is_empty() {
        return Ok(Vec::new());
    }

    let mut args = vec![
        "log".to_string(),
        "--no-walk=unsorted".to_string(),
        "--format=%H%x1f%h%x1f%s%x1e".to_string(),
    ];
    args.extend(hashes.iter().cloned());
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = git_text(repo_path, &arg_refs)?;

    Ok(output
        .split('\u{1e}')
        .filter_map(parse_step_push_plan_item)
        .collect())
}

pub fn validate_push_target(repo_path: &str, hash: &str) -> AppResult<()> {
    let repo_path = resolve_repository_path(repo_path)?;
    ensure_safe_push_target(&repo_path, hash)
}

pub fn validate_step_push_hashes(repo_path: &str, hashes: &[String]) -> AppResult<()> {
    let repo_path = resolve_repository_path(repo_path)?;
    ensure_safe_step_push_hashes(&repo_path, hashes)
}

pub fn get_step_push_plan(repo_path: &str, target_hash: &str) -> AppResult<StepPushPlan> {
    let repo_path = resolve_repository_path(repo_path)?;
    let target_hash = target_hash.trim();

    if target_hash.is_empty() {
        return Err(AppError::new("invalid_hash", "目标 Commit 不能为空"));
    }

    let sync_started_at = Instant::now();
    match sync_origin_tracking(&repo_path) {
        Ok(()) => log_step_push_plan_stage(
            "sync_origin_tracking",
            sync_started_at,
            target_hash,
            None,
            None,
            "ok",
        ),
        Err(error) => {
            log_step_push_plan_stage(
                "sync_origin_tracking",
                sync_started_at,
                target_hash,
                None,
                None,
                "error",
            );
            return Err(error);
        }
    }

    let branch_status_started_at = Instant::now();
    let branch_status = match branch_status_for_path(&repo_path) {
        Ok(branch_status) => branch_status,
        Err(error) => {
            log_step_push_plan_stage(
                "branch_status_for_path",
                branch_status_started_at,
                target_hash,
                None,
                None,
                "error",
            );
            return Err(error);
        }
    };
    let upstream = branch_status.upstream.as_deref();
    if let Some(reason) = step_push_branch_blocked_reason(&branch_status) {
        log_step_push_plan_stage(
            "branch_status_for_path",
            branch_status_started_at,
            target_hash,
            Some(branch_status.branch.as_str()),
            upstream,
            "blocked",
        );
        return Ok(blocked_step_push_plan(
            branch_status.branch,
            target_hash.to_string(),
            reason.code,
            reason.message,
        ));
    }
    log_step_push_plan_stage(
        "branch_status_for_path",
        branch_status_started_at,
        target_hash,
        Some(branch_status.branch.as_str()),
        upstream,
        "ok",
    );

    let upstream = upstream.ok_or_else(|| {
        AppError::new("push_unavailable", "当前分支没有上游跟踪分支，无法执行推送")
    })?;

    let safe_hashes_started_at = Instant::now();
    let safe_hashes = match safe_unpushed_hashes_in_push_order(&repo_path, upstream) {
        Ok(safe_hashes) => safe_hashes,
        Err(error) => {
            log_step_push_plan_stage(
                "safe_unpushed_hashes_in_push_order",
                safe_hashes_started_at,
                target_hash,
                Some(branch_status.branch.as_str()),
                Some(upstream),
                "error",
            );
            return Err(error);
        }
    };
    let Some(target_index) = safe_hashes.iter().position(|hash| hash == target_hash) else {
        log_step_push_plan_stage(
            "safe_unpushed_hashes_in_push_order",
            safe_hashes_started_at,
            target_hash,
            Some(branch_status.branch.as_str()),
            Some(upstream),
            "blocked",
        );
        return Ok(blocked_step_push_plan(
            branch_status.branch,
            target_hash.to_string(),
            "unsafe_push_target",
            UNSAFE_PUSH_TARGET_MESSAGE,
        ));
    };
    log_step_push_plan_stage(
        "safe_unpushed_hashes_in_push_order",
        safe_hashes_started_at,
        target_hash,
        Some(branch_status.branch.as_str()),
        Some(upstream),
        "ok",
    );

    let ensure_remote_started_at = Instant::now();
    if let Err(error) =
        ensure_remote_fast_forward_target(&repo_path, &branch_status.branch, target_hash)
    {
        if error.code == "push_unavailable" {
            log_step_push_plan_stage(
                "ensure_remote_fast_forward_target",
                ensure_remote_started_at,
                target_hash,
                Some(branch_status.branch.as_str()),
                Some(upstream),
                "blocked",
            );
            return Ok(blocked_step_push_plan(
                branch_status.branch,
                target_hash.to_string(),
                "behind_remote",
                BRANCH_BEHIND_REMOTE_MESSAGE,
            ));
        }

        log_step_push_plan_stage(
            "ensure_remote_fast_forward_target",
            ensure_remote_started_at,
            target_hash,
            Some(branch_status.branch.as_str()),
            Some(upstream),
            "error",
        );
        return Err(error);
    }
    log_step_push_plan_stage(
        "ensure_remote_fast_forward_target",
        ensure_remote_started_at,
        target_hash,
        Some(branch_status.branch.as_str()),
        Some(upstream),
        "ok",
    );

    let items_started_at = Instant::now();
    let items = match step_push_plan_items(&repo_path, &safe_hashes[..=target_index]) {
        Ok(items) => items,
        Err(error) => {
            log_step_push_plan_stage(
                "step_push_plan_items",
                items_started_at,
                target_hash,
                Some(branch_status.branch.as_str()),
                Some(upstream),
                "error",
            );
            return Err(error);
        }
    };
    log_step_push_plan_stage(
        "step_push_plan_items",
        items_started_at,
        target_hash,
        Some(branch_status.branch.as_str()),
        Some(upstream),
        "ok",
    );

    Ok(StepPushPlan {
        branch: branch_status.branch,
        target_hash: target_hash.to_string(),
        available: true,
        items,
        blocked_reason: None,
    })
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

pub fn push_to_commit_prechecked(repo_path: &str, branch: &str, hash: &str) -> AppResult<()> {
    let repo_path = resolve_repository_path(repo_path)?;
    let branch = current_branch_matching(&repo_path, branch)?;
    let refspec = format!("{hash}:refs/heads/{branch}");
    git_run(&repo_path, &["push", REMOTE_NAME, &refspec])
}
