export type RepositorySummary = {
  name: string;
  path: string;
  lastOpenedAt: number;
};

export type BranchStatus = {
  branch: string;
  upstream: string | null;
  aheadCount: number;
  behindCount: number;
  detached: boolean;
  pushAvailable: boolean;
  disabledReason: string | null;
};

export type CommitListItem = {
  hash: string;
  shortHash: string;
  summary: string;
  authorName: string;
  authorEmail: string;
  committedAt: string;
  tags: string[];
  parents: string[];
  isPushed: boolean;
};

export type CommitMeta = {
  body: string;
  additions: number;
  deletions: number;
};

export type CommitHistoryPage = {
  items: CommitListItem[];
  nextSkip: number;
  hasMore: boolean;
  unpushedCount: number;
};

export type CommitFileChange = {
  status: string;
  path: string;
  previousPath: string | null;
  displayPath: string;
};

export type StepPushRequest = {
  repoPath: string;
  branch: string;
  hashes: string[];
  delayMs?: number;
};

export type PushToCommitRequest = {
  repoPath: string;
  branch: string;
  hash: string;
};

export type PushTargetKind = "commit" | "branch";

export type PushToCommitJobStarted = {
  jobId: number;
  target: string;
  targetKind: PushTargetKind;
};

export type PushToCommitFinished = {
  jobId: number;
  target: string;
  targetKind: PushTargetKind;
};

export type PushToCommitFailed = {
  jobId: number;
  target: string;
  targetKind: PushTargetKind;
  message: string;
};

export type StepPushJobStarted = {
  jobId: number;
  total: number;
};

export type StepPushProgress = {
  jobId: number;
  current: number;
  total: number;
  hash: string;
  status: "running";
};

export type StepPushFinished = {
  jobId: number;
  total: number;
};

export type StepPushFailed = {
  jobId: number;
  current: number;
  total: number;
  hash: string;
  message: string;
};

export type AppError = {
  code: string;
  message: string;
};

export type ToastItem = {
  id: number;
  title: string;
  message: string;
  tone?: "info" | "success" | "error";
};

export type StepPushUiState = {
  jobId: number;
  current: number;
  total: number;
  hash: string;
  status: "running" | "finished" | "failed";
  message?: string;
};

export type PushToCommitUiState = {
  jobId: number;
  target: string;
  targetKind: PushTargetKind;
  status: "running" | "finished" | "failed";
  message?: string;
};
