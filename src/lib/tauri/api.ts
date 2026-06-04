import { invoke } from "@tauri-apps/api/core";
import type {
  BranchStatus,
  CommitMeta,
  CommitFileChange,
  CommitFileDiffResult,
  CommitHistoryPage,
  PushToCommitJobStarted,
  PushToCommitRequest,
  RepositorySummary,
  StepPushJobStarted,
  StepPushRequest,
} from "$lib/types";

export const api = {
  listRepositories: () => invoke<RepositorySummary[]>("list_repositories"),
  addRepository: (path: string) =>
    invoke<RepositorySummary>("add_repository", { path }),
  setCurrentRepository: (path: string) =>
    invoke<void>("set_current_repository", { path }),
  getCurrentRepository: () =>
    invoke<RepositorySummary | null>("get_current_repository"),
  getBranchStatus: (repoPath: string) =>
    invoke<BranchStatus>("get_branch_status", { repoPath }),
  refreshRemoteTracking: (repoPath: string) =>
    invoke<void>("refresh_remote_tracking", { repoPath }),
  listLocalBranches: (repoPath: string) =>
    invoke<string[]>("list_local_branches", { repoPath }),
  checkoutBranch: (repoPath: string, branch: string) =>
    invoke<void>("checkout_branch", { repoPath, branch }),
  getCommitHistory: (repoPath: string, skip: number, limit: number) =>
    invoke<CommitHistoryPage>("get_commit_history", { repoPath, skip, limit }),
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
  pushCurrentBranch: (repoPath: string) =>
    invoke<void>("push_current_branch", { repoPath }),
  startPushCurrentBranch: (repoPath: string, branch: string) =>
    invoke<PushToCommitJobStarted>("start_push_current_branch", {
      repoPath,
      branch,
    }),
  saveWindowSize: (width: number, height: number) =>
    invoke<void>("save_window_size", { width, height }),
  pushToCommit: (repoPath: string, branch: string, hash: string) =>
    invoke<void>("push_to_commit", { repoPath, branch, hash }),
  startPushToCommit: (request: PushToCommitRequest) =>
    invoke<PushToCommitJobStarted>("start_push_to_commit", { request }),
  startStepPush: (request: StepPushRequest) =>
    invoke<StepPushJobStarted>("start_step_push", { request }),
};
