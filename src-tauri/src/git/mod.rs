mod command;
mod diff;
mod history;
mod parse;
mod push;
mod rebase;
mod repository;
mod stash;

pub use diff::get_commit_file_diff;
pub use history::{get_commit_files, get_commit_history, get_commit_meta};
pub use push::{
    get_step_push_plan, push_current_branch_checked, push_to_commit, push_to_commit_prechecked,
    validate_push_target, validate_step_push_hashes,
};
pub use rebase::rebase_branch_onto_upstream;
pub use repository::{
    checkout_branch, get_branch_status, get_repository_revision, list_local_branches,
    refresh_remote_tracking, resolve_repository_path, validate_current_branch,
};
pub use stash::{get_stash_state, restore_worktree_stash, stash_current_worktree};

const REMOTE_NAME: &str = "origin";
const UNSAFE_PUSH_TARGET_MESSAGE: &str =
    "该 Commit 不在 TickGit 的安全推送路径上。若它不包含远端最新提交，Git 会拒绝单独推送。想分批推送这些本地提交，需要先将它们 rebase/cherry-pick 到远端最新提交之后；这会重写 Commit ID，也可能产生冲突。若要保留现有历史，可以直接推送当前分支。";
const BRANCH_BEHIND_REMOTE_MESSAGE: &str =
    "远端已有更新，TickGit 暂不能安全推送。请先使用 GitHub Desktop 或 SourceTree 同步远端后，再回到 TickGit 刷新重试。";
const BRANCH_MISMATCH_MESSAGE: &str = "目标分支与当前检出分支不一致，已拒绝推送";

// UI 的 ahead / 历史列表采用完整 upstream..HEAD 口径，尽量贴近 GitHub Desktop；
// 分步推送和 push to commit 仍只允许 first-parent 安全路径，避免 merge 侧支被当成线性推送目标。

#[cfg(test)]
mod tests;
