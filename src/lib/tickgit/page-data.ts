import type {
  BranchStatus,
  CommitMeta,
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
  getCommitMeta: (repoPath: string, hash: string) => Promise<CommitMeta>;
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
  commitMeta: CommitMeta | null;
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
  const [commitFiles, commitMeta] = await Promise.all([
    api.getCommitFiles(repoPath, hash),
    api.getCommitMeta(repoPath, hash),
  ]);
  const selectedFilePath = commitFiles[0]?.path ?? null;
  // 没有文件变更时无需再请求 diff；否则既浪费一次 invoke，也会让空详情路径变得不明确。
  const diffText = selectedFilePath
    ? await api.getCommitFileDiff(repoPath, hash, selectedFilePath)
    : "";

  return {
    commitFiles,
    commitMeta,
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
    // aheadCount 可能大于第一页大小；这里预先补齐全部未推送 commit，避免右键推送/分步推送只拿到局部列表。
    branchStatus.aheadCount > 0 &&
    commits.length < branchStatus.aheadCount &&
    hasMore
  );

  // 先稳定选中项，再去拉详情；这样刷新后才能正确保留旧选中，或在选中丢失时回退到首项。
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
      commitMeta: null,
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
    commitMeta: details.commitMeta,
    commitFiles: details.commitFiles,
    selectedFilePath: details.selectedFilePath,
    diffText: details.diffText,
  };
}
