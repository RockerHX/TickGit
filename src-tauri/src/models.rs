use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySummary {
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
    pub behind_count: usize,
    pub detached: bool,
    pub push_available: bool,
    pub disabled_reason: Option<String>,
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
    pub unpushed_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommitFileChange {
    pub status: String,
    pub path: String,
    pub previous_path: Option<String>,
    pub display_path: String,
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
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryConfig {
    pub repositories: Vec<RepositorySummary>,
    pub current_path: Option<String>,
    pub window_size: Option<WindowSizeConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WindowSizeConfig {
    pub width: f64,
    pub height: f64,
}
