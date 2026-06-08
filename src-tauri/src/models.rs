use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySummary {
    pub name: String,
    pub path: String,
    pub last_opened_at: i64,
    pub status: RepositoryStatus,
    pub disabled_reason: Option<String>,
    pub disabled_reason_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RepositoryStatus {
    Available,
    Missing,
    Invalid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StoredRepository {
    pub name: String,
    pub path: String,
    pub last_opened_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BranchStatus {
    pub branch: String,
    pub upstream: Option<String>,
    pub ahead_count: usize,
    pub safe_ahead_count: usize,
    pub behind_count: usize,
    pub detached: bool,
    pub push_available: bool,
    pub disabled_reason: Option<String>,
    pub disabled_reason_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommitListItem {
    pub hash: String,
    pub short_hash: String,
    pub summary: String,
    pub author_name: String,
    pub author_email: String,
    pub committed_at: String,
    pub tags: Vec<String>,
    pub parents: Vec<String>,
    pub is_pushed: bool,
    pub is_safe_push_target: bool,
    pub push_blocked_reason: Option<String>,
    pub push_blocked_reason_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommitMeta {
    pub body: String,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommitHistoryPage {
    pub items: Vec<CommitListItem>,
    pub next_skip: usize,
    pub has_more: bool,
    pub total_count: usize,
    pub unpushed_count: usize,
    pub safe_unpushed_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct CommitHistoryFilters {
    pub query: Option<String>,
    pub author: Option<String>,
    pub file_path: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommitFileChange {
    pub status: String,
    pub path: String,
    pub previous_path: Option<String>,
    pub display_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additions: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deletions: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommitFileDiffResult {
    pub text: String,
    pub is_binary: bool,
    pub is_image: bool,
    pub is_too_large: bool,
    pub truncated: bool,
    pub byte_count: usize,
    pub line_count: usize,
    pub old_image_data_url: Option<String>,
    pub new_image_data_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StepPushPlan {
    pub branch: String,
    pub target_hash: String,
    pub available: bool,
    pub items: Vec<StepPushPlanItem>,
    pub blocked_reason: Option<StepPushPlanBlockedReason>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StepPushPlanItem {
    pub hash: String,
    pub short_hash: String,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StepPushPlanBlockedReason {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StepPushRequest {
    pub repo_path: String,
    pub branch: String,
    pub hashes: Vec<String>,
    pub delay_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StepPushJobStarted {
    pub job_id: u64,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StepPushProgress {
    pub job_id: u64,
    pub current: usize,
    pub total: usize,
    pub hash: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StepPushFinished {
    pub job_id: u64,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StepPushFailed {
    pub job_id: u64,
    pub current: usize,
    pub total: usize,
    pub hash: String,
    pub message: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PushToCommitRequest {
    pub repo_path: String,
    pub branch: String,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum PushTargetKind {
    Commit,
    Branch,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PushToCommitJobStarted {
    pub job_id: u64,
    pub target: String,
    pub target_kind: PushTargetKind,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PushToCommitFinished {
    pub job_id: u64,
    pub target: String,
    pub target_kind: PushTargetKind,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PushToCommitFailed {
    pub job_id: u64,
    pub target: String,
    pub target_kind: PushTargetKind,
    pub message: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryConfig {
    pub repositories: Vec<StoredRepository>,
    pub current_path: Option<String>,
    pub window_size: Option<WindowSizeConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WindowSizeConfig {
    pub width: f64,
    pub height: f64,
}
