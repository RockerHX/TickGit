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
  parents: string[];
  isPushed: boolean;
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
