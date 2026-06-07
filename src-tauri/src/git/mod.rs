mod command;
mod diff;
mod history;
mod parse;
mod push;
mod repository;

pub use diff::get_commit_file_diff;
pub use history::{get_commit_files, get_commit_history, get_commit_meta};
pub use push::{
    get_step_push_plan, push_current_branch_checked, push_to_commit, validate_push_target,
    validate_step_push_hashes,
};
pub use repository::{
    checkout_branch, get_branch_status, list_local_branches, refresh_remote_tracking,
    resolve_repository_path, validate_current_branch,
};

const REMOTE_NAME: &str = "origin";
const UNSAFE_PUSH_TARGET_MESSAGE: &str =
    "该 Commit 未推送，但不在 first-parent 安全路径上，不能作为 step push / push to commit 目标";
const BRANCH_BEHIND_REMOTE_MESSAGE: &str =
    "远端已有更新，TickGit 暂不能安全推送。请先使用 GitHub Desktop 或 SourceTree 同步远端后，再回到 TickGit 刷新重试。";
const BRANCH_MISMATCH_MESSAGE: &str = "目标分支与当前检出分支不一致，已拒绝推送";

// UI 的 ahead / 历史列表采用完整 upstream..HEAD 口径，尽量贴近 GitHub Desktop；
// 分步推送和 push to commit 仍只允许 first-parent 安全路径，避免 merge 侧支被当成线性推送目标。

#[cfg(test)]
mod tests;
