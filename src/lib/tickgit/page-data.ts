import type {
  BranchStatus,
  CommitFileChange,
  CommitHistoryPage,
  CommitListItem,
  RepositorySummary,
} from "$lib/types";
import { pickSelectedCommit } from "$lib/tickgit/page-helpers";

export type TickGitPageApi = {
  listRepositories: () => Promise<RepositorySummary[]>;
  getCurrentRepository: () => Promise<RepositorySummary | null>;
  getBranchStatus: (repoPath: string) => Promise<BranchStatus>;
  getCommitHistory: (
    repoPath: string,
    skip: number,
    limit: number,
  ) => Promise<CommitHistoryPage>;
  getCommitFiles: (
    repoPath: string,
    hash: string,
  ) => Promise<CommitFileChange[]>;
  getCommitFileDiff: (
    repoPath: string,
    hash: string,
    filePath: string,
  ) => Promise<string>;
};

export type RepositorySnapshot = {
  branchStatus: BranchStatus;
  commits: CommitListItem[];
  nextSkip: number;
  hasMore: boolean;
  selectedCommit: CommitListItem | null;
  commitFiles: CommitFileChange[];
  selectedFilePath: string | null;
  diffText: string;
};

export async function fetchRepositoryIndex(api: TickGitPageApi) {
  const [repositories, currentRepository] = await Promise.all([
    api.listRepositories(),
    api.getCurrentRepository(),
  ]);

  return { repositories, currentRepository };
}

export async function fetchCommitDetails(
  api: TickGitPageApi,
  repoPath: string,
  hash: string,
) {
  const commitFiles = await api.getCommitFiles(repoPath, hash);
  const selectedFilePath = commitFiles[0]?.path ?? null;
  const diffText = selectedFilePath
    ? await api.getCommitFileDiff(repoPath, hash, selectedFilePath)
    : "";

  return {
    commitFiles,
    selectedFilePath,
    diffText,
  };
}

export async function fetchRepositorySnapshot(
  api: TickGitPageApi,
  repoPath: string,
  pageSize: number,
  keepSelection: boolean,
  previousSelectedHash: string | null,
): Promise<RepositorySnapshot> {
  const branchStatus = await api.getBranchStatus(repoPath);

  let commits: CommitListItem[] = [];
  let nextSkip = 0;
  let hasMore = false;

  do {
    const page = await api.getCommitHistory(repoPath, nextSkip, pageSize);
    commits = [...commits, ...page.items];
    nextSkip = page.nextSkip;
    hasMore = page.hasMore;
  } while (
    branchStatus.aheadCount > 0 &&
    commits.length < branchStatus.aheadCount &&
    hasMore
  );

  const selectedCommit = pickSelectedCommit(
    commits,
    previousSelectedHash,
    keepSelection,
  );

  if (!selectedCommit) {
    return {
      branchStatus,
      commits,
      nextSkip,
      hasMore,
      selectedCommit: null,
      commitFiles: [],
      selectedFilePath: null,
      diffText: "",
    };
  }

  const details = await fetchCommitDetails(api, repoPath, selectedCommit.hash);

  return {
    branchStatus,
    commits,
    nextSkip,
    hasMore,
    selectedCommit,
    commitFiles: details.commitFiles,
    selectedFilePath: details.selectedFilePath,
    diffText: details.diffText,
  };
}
