import { invoke } from "@tauri-apps/api/core";
import type {
  BranchStatus,
  CommitMeta,
  CommitFileChange,
  CommitFileDiffResult,
  CommitDetails,
  CommitHistoryFilters,
  CommitHistoryPage,
  PushToCommitJobStarted,
  PushToCommitRequest,
  RepositoryIndex,
  RepositoryOverview,
  RepositoryOverviewCacheEntry,
  RepositoryStatusUpdate,
  RepositorySummary,
  StepPushJobStarted,
  StepPushPlan,
  StepPushRequest,
} from "$lib/types";

export const api = {
  getRepositoryIndexFast: () =>
    invoke<RepositoryIndex>("get_repository_index_fast"),
  getCachedRepositoryOverview: () =>
    invoke<RepositoryOverviewCacheEntry | null>(
      "get_cached_repository_overview",
    ),
  refreshRepositoryStatuses: (paths: string[]) =>
    invoke<RepositoryStatusUpdate[]>("refresh_repository_statuses", { paths }),
  getRepositoryOverview: (
    repoPath: string,
    skip: number,
    limit: number,
    filters?: CommitHistoryFilters | null,
  ) =>
    invoke<RepositoryOverview>("get_repository_overview", {
      repoPath,
      skip,
      limit,
      filters: filters ?? null,
    }),
  getCommitDetails: (repoPath: string, hash: string) =>
    invoke<CommitDetails>("get_commit_details", { repoPath, hash }),
  listRepositories: () => invoke<RepositorySummary[]>("list_repositories"),
  addRepository: (path: string) =>
    invoke<RepositorySummary>("add_repository", { path }),
  setCurrentRepository: (path: string) =>
    invoke<void>("set_current_repository", { path }),
  getCurrentRepository: () =>
    invoke<RepositorySummary | null>("get_current_repository"),
  removeRepository: (path: string) =>
    invoke<RepositorySummary | null>("remove_repository", { path }),
  relocateRepository: (oldPath: string, newPath: string) =>
    invoke<RepositorySummary>("relocate_repository", { oldPath, newPath }),
  getBranchStatus: (repoPath: string) =>
    invoke<BranchStatus>("get_branch_status", { repoPath }),
  refreshRemoteTracking: (repoPath: string) =>
    invoke<void>("refresh_remote_tracking", { repoPath }),
  listLocalBranches: (repoPath: string) =>
    invoke<string[]>("list_local_branches", { repoPath }),
  checkoutBranch: (repoPath: string, branch: string) =>
    invoke<void>("checkout_branch", { repoPath, branch }),
  getCommitHistory: (
    repoPath: string,
    skip: number,
    limit: number,
    filters?: CommitHistoryFilters | null,
  ) =>
    invoke<CommitHistoryPage>("get_commit_history", {
      repoPath,
      skip,
      limit,
      filters: filters ?? null,
    }),
  getCommitFiles: (repoPath: string, hash: string) =>
    invoke<CommitFileChange[]>("get_commit_files", { repoPath, hash }),
  getCommitMeta: (repoPath: string, hash: string) =>
    invoke<CommitMeta>("get_commit_meta", { repoPath, hash }),
  getCommitFileDiff: (
    repoPath: string,
    hash: string,
    filePath: string,
    ignoreWhitespace = false,
    previousPath?: string | null,
  ) =>
    invoke<CommitFileDiffResult>("get_commit_file_diff", {
      repoPath,
      hash,
      filePath,
      previousPath: previousPath ?? null,
      ignoreWhitespace,
    }),
  startPushCurrentBranch: (repoPath: string, branch: string) =>
    invoke<PushToCommitJobStarted>("start_push_current_branch", {
      repoPath,
      branch,
    }),
  saveWindowSize: (width: number, height: number) =>
    invoke<void>("save_window_size", { width, height }),
  getStepPushPlan: (repoPath: string, targetHash: string) =>
    invoke<StepPushPlan>("get_step_push_plan", { repoPath, targetHash }),
  startPushToCommit: (request: PushToCommitRequest) =>
    invoke<PushToCommitJobStarted>("start_push_to_commit", { request }),
  startStepPush: (request: StepPushRequest) =>
    invoke<StepPushJobStarted>("start_step_push", { request }),
};
