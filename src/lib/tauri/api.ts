import { invoke } from "@tauri-apps/api/core";
import type {
  BranchStatus,
  CommitFileChange,
  CommitHistoryPage,
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
  getBranchStatus: (repoPath: string, branch?: string | null) =>
    invoke<BranchStatus>("get_branch_status", { repoPath, branch }),
  getCommitHistory: (repoPath: string, skip: number, limit: number) =>
    invoke<CommitHistoryPage>("get_commit_history", { repoPath, skip, limit }),
  getCommitFiles: (repoPath: string, hash: string) =>
    invoke<CommitFileChange[]>("get_commit_files", { repoPath, hash }),
  getCommitFileDiff: (repoPath: string, hash: string, filePath: string) =>
    invoke<string>("get_commit_file_diff", { repoPath, hash, filePath }),
  pushCurrentBranch: (repoPath: string) =>
    invoke<void>("push_current_branch", { repoPath }),
  pushToCommit: (repoPath: string, branch: string, hash: string) =>
    invoke<void>("push_to_commit", { repoPath, branch, hash }),
  startStepPush: (request: StepPushRequest) =>
    invoke<StepPushJobStarted>("start_step_push", { request }),
};
