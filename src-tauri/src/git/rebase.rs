use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    error::{AppError, AppResult},
    models::RebaseResult,
};

use super::{
    command::{git_run, git_trimmed, git_trimmed_allow_exit_code},
    push::ensure_safe_push_target,
    repository::{
        branch_status_for_path, current_branch_matching, resolve_repository_path,
        sync_origin_tracking, upstream_is_origin, upstream_name,
    },
    stash::{ensure_no_git_operation_in_progress, restore_stash, stash_worktree_changes},
};

fn create_backup_branch(repo_path: &Path, head: &str) -> AppResult<String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();

    for attempt in 0..100 {
        let name = format!("tickgit-backup-{timestamp}-{attempt}");
        let reference = format!("refs/heads/{name}");
        let existing = git_trimmed_allow_exit_code(
            repo_path,
            &["rev-parse", "--verify", "--quiet", &reference],
            1,
        )?;

        if existing.is_empty() {
            git_run(repo_path, &["branch", &name, head])?;
            return Ok(name);
        }
    }

    Err(AppError::new(
        "rebase_backup_failed",
        "无法创建唯一的临时备份分支。",
    ))
}

fn rebase_failure_with_backup(repo_path: &Path, backup_branch: &str, error: AppError) -> AppError {
    match git_run(repo_path, &["rebase", "--abort"]) {
        Ok(()) => AppError::new(
            "rebase_conflict",
            format!(
                "整理提交失败，Git 已撤销本次 rebase。备份分支 {backup_branch} 已保留。{}",
                error.message
            ),
        ),
        Err(abort_error) => AppError::new(
            "rebase_abort_failed",
            format!(
                "整理提交失败，且无法自动撤销 rebase。备份分支 {backup_branch} 已保留。请在外部 Git 工具中解决或取消 rebase。{} {}",
                error.message, abort_error.message
            ),
        ),
    }
}

fn restore_backup_head(repo_path: &Path, backup_branch: &str) -> AppResult<()> {
    git_run(repo_path, &["reset", "--hard", backup_branch]).map_err(|error| {
        AppError::new(
            "rebase_restore_failed",
            format!(
                "无法恢复原分支。备份分支 {backup_branch} 已保留，请勿删除。{}",
                error.message
            ),
        )
    })
}

fn restore_stash_after_failure(
    repo_path: &Path,
    backup_branch: &str,
    stash_hash: Option<&str>,
    error: AppError,
) -> AppError {
    let Some(stash_hash) = stash_hash else {
        return error;
    };

    if error.code == "rebase_abort_failed" {
        return AppError::new(
            "rebase_stash_restore_failed",
            format!(
                "{} 工作区改动仍保存在 stash {stash_hash}；备份分支 {backup_branch} 已保留。请先在外部 Git 工具中完成或取消 rebase。",
                error.message
            ),
        );
    }

    match restore_stash(repo_path, stash_hash) {
        Ok(()) => error,
        Err(stash_error) => AppError::new(
            "rebase_stash_restore_failed",
            format!(
                "{} 工作区改动未能自动恢复，仍保存在 stash {stash_hash}；备份分支 {backup_branch} 已保留。{}",
                error.message, stash_error.message
            ),
        ),
    }
}

fn restore_original_and_stash(
    repo_path: &Path,
    backup_branch: &str,
    stash_hash: Option<&str>,
    error: AppError,
) -> AppError {
    if let Err(restore_error) = restore_backup_head(repo_path, backup_branch) {
        let stash_note = stash_hash.map_or_else(String::new, |hash| {
            format!("工作区改动仍保存在 stash {hash}。")
        });
        return AppError::new(
            "rebase_restore_failed",
            format!(
                "{} 恢复原分支失败。备份分支 {backup_branch} 已保留。{}{}",
                error.message, stash_note, restore_error.message
            ),
        );
    }

    let error = AppError::new(error.code, format!("{} 已恢复原分支。", error.message));
    restore_stash_after_failure(repo_path, backup_branch, stash_hash, error)
}

pub fn rebase_branch_onto_upstream(repo_path: &str, branch: &str) -> AppResult<RebaseResult> {
    let repo_path = resolve_repository_path(repo_path)?;
    let repo_path = PathBuf::from(git_trimmed(&repo_path, &["rev-parse", "--show-toplevel"])?);
    let branch = current_branch_matching(&repo_path, branch)?;

    sync_origin_tracking(&repo_path)?;
    ensure_no_git_operation_in_progress(&repo_path)?;

    let upstream = upstream_name(&repo_path).ok_or_else(|| {
        AppError::new(
            "missing_upstream",
            "当前分支没有上游跟踪分支，无法整理提交。",
        )
    })?;
    if !upstream_is_origin(&upstream) {
        return Err(AppError::new(
            "non_origin_upstream",
            "当前分支的上游不是 origin，无法整理提交。",
        ));
    }
    if upstream != format!("origin/{branch}") {
        return Err(AppError::new(
            "rebase_upstream_mismatch",
            "当前分支的上游不是 origin 上的同名分支，无法安全整理提交。",
        ));
    }

    let status = branch_status_for_path(&repo_path)?;
    if status.ahead_count == 0 {
        return Err(AppError::new(
            "rebase_no_commits",
            "当前分支没有未推送的本地提交。",
        ));
    }
    if status.behind_count > 0 {
        return Err(AppError::new(
            "rebase_remote_advanced",
            "origin 在上次同步后又有新提交。请先在 GitHub Desktop 或其他 Git 工具中同步并解决冲突，再重新整理。",
        ));
    }
    if status.safe_ahead_count == status.ahead_count {
        return Err(AppError::new(
            "rebase_not_needed",
            "当前未推送提交已经可以安全分步推送，无需整理。",
        ));
    }

    let original_head = git_trimmed(&repo_path, &["rev-parse", "HEAD"])?;
    let original_tree = git_trimmed(
        &repo_path,
        &["rev-parse", &format!("{original_head}^{{tree}}")],
    )?;
    let backup_branch = create_backup_branch(&repo_path, &original_head)?;
    let stash_hash = match stash_worktree_changes(&repo_path, &backup_branch, &branch) {
        Ok(stash_hash) => stash_hash,
        Err(error) => {
            return Err(AppError::new(
                error.code,
                format!(
                    "本次未开始 rebase。备份分支 {backup_branch} 已保留。{}",
                    error.message
                ),
            ));
        }
    };

    if let Err(error) = git_run(
        &repo_path.as_path(),
        &[
            "-c",
            "rebase.updateRefs=false",
            "rebase",
            "--no-autosquash",
            &upstream,
        ],
    ) {
        let error = rebase_failure_with_backup(&repo_path, &backup_branch, error);
        return Err(restore_stash_after_failure(
            &repo_path,
            &backup_branch,
            stash_hash.as_deref(),
            error,
        ));
    }

    let rebased_head = git_trimmed(&repo_path, &["rev-parse", "HEAD"])?;
    let rebased_tree = git_trimmed(
        &repo_path,
        &["rev-parse", &format!("{rebased_head}^{{tree}}")],
    )?;
    if original_tree != rebased_tree {
        let error = AppError::new(
            "rebase_tree_changed",
            format!(
                "rebase 后文件内容与原合并结果不同，可能包含合并时手动解决冲突的修改。备份分支 {backup_branch} 已保留，请检查差异并把需要保留的内容单独提交。"
            ),
        );
        return Err(restore_original_and_stash(
            &repo_path,
            &backup_branch,
            stash_hash.as_deref(),
            error,
        ));
    }

    if let Err(error) = ensure_safe_push_target(&repo_path, &rebased_head) {
        let error = AppError::new(
            "rebase_verification_failed",
            format!(
                "整理后未能确认目标可以安全推送。备份分支 {backup_branch} 已保留。{}",
                error.message
            ),
        );
        return Err(restore_original_and_stash(
            &repo_path,
            &backup_branch,
            stash_hash.as_deref(),
            error,
        ));
    }

    let rebased_status = match branch_status_for_path(&repo_path) {
        Ok(status)
            if status.behind_count == 0
                && status.ahead_count > 0
                && status.safe_ahead_count == status.ahead_count =>
        {
            status
        }
        Ok(_) => {
            let error = AppError::new(
                "rebase_verification_failed",
                format!("整理后未能验证所有提交都可分步推送。备份分支 {backup_branch} 已保留。"),
            );
            return Err(restore_original_and_stash(
                &repo_path,
                &backup_branch,
                stash_hash.as_deref(),
                error,
            ));
        }
        Err(error) => {
            let error = AppError::new(
                "rebase_verification_failed",
                format!(
                    "无法验证整理后的提交。备份分支 {backup_branch} 已保留。{}",
                    error.message
                ),
            );
            return Err(restore_original_and_stash(
                &repo_path,
                &backup_branch,
                stash_hash.as_deref(),
                error,
            ));
        }
    };

    if let Some(stash_hash) = stash_hash.as_deref() {
        if let Err(error) = restore_stash(&repo_path, stash_hash) {
            return Err(AppError::new(
                "rebase_stash_restore_failed",
                format!(
                    "提交历史已整理并通过推送检查，但工作区改动未能自动恢复，仍保存在 stash {stash_hash}。备份分支 {backup_branch} 已保留。{}",
                    error.message
                ),
            ));
        }
    }

    let backup_branch_deleted = git_run(&repo_path, &["branch", "-D", &backup_branch]).is_ok();

    Ok(RebaseResult {
        branch,
        rebased_commit_count: rebased_status.ahead_count,
        backup_branch,
        backup_branch_deleted,
    })
}
