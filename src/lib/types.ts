export type RepositoryStatus = "available" | "missing" | "invalid";

export type RepositorySummary = {
  name: string;
  path: string;
  lastOpenedAt: number;
  status: RepositoryStatus;
  disabledReason: string | null;
  disabledReasonCode?: string | null;
};

export type BranchStatus = {
  branch: string;
  upstream: string | null;
  aheadCount: number;
  safeAheadCount: number;
  behindCount: number;
  detached: boolean;
  pushAvailable: boolean;
  disabledReason: string | null;
  disabledReasonCode?: string | null;
};

export type RepositoryRevision = {
  head: string;
  branch: string;
  upstream: string | null;
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
  isSafePushTarget: boolean;
  pushBlockedReason: string | null;
  pushBlockedReasonCode?: string | null;
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
  totalCount: number;
  unpushedCount: number;
  safeUnpushedCount: number;
};

export type CommitHistoryFilters = {
  query?: string | null;
  author?: string | null;
  filePath?: string | null;
  message?: string | null;
};

export type CommitFileChange = {
  status: string;
  path: string;
  previousPath: string | null;
  displayPath: string;
  additions?: number;
  deletions?: number;
  language?: string | null;
};

export type CommitFileDiffResult = {
  text: string;
  isBinary: boolean;
  isImage: boolean;
  isTooLarge: boolean;
  truncated: boolean;
  byteCount: number;
  lineCount: number;
  oldImageDataUrl: string | null;
  newImageDataUrl: string | null;
};

export type StepPushPlan = {
  branch: string;
  targetHash: string;
  available: boolean;
  items: StepPushPlanItem[];
  blockedReason: StepPushPlanBlockedReason | null;
};

export type StepPushPlanItem = {
  hash: string;
  shortHash: string;
  summary: string;
};

export type StepPushPlanBlockedReason = {
  code: string;
  message: string;
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
  code?: string | null;
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
  code?: string | null;
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
  code?: string | null;
};

export type PushToCommitUiState = {
  jobId: number;
  target: string;
  targetKind: PushTargetKind;
  status: "running" | "finished" | "failed";
  message?: string;
  code?: string | null;
};
