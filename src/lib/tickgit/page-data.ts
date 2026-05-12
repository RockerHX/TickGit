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
    ignoreWhitespace?: boolean,
    previousPath?: string | null,
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
  ignoreWhitespace = false,
) {
  const [commitFiles, commitMeta] = await Promise.all([
    api.getCommitFiles(repoPath, hash),
    api.getCommitMeta(repoPath, hash),
  ]);
  const selectedFile = commitFiles[0] ?? null;
  const selectedFilePath = selectedFile?.path ?? null;
  // 没有文件变更时无需再请求 diff；否则既浪费一次 invoke，也会让空详情路径变得不明确。
  const diffText = selectedFile
    ? await api.getCommitFileDiff(
        repoPath,
        hash,
        selectedFile.path,
        ignoreWhitespace,
        selectedFile.previousPath,
      )
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
  ignoreWhitespace = false,
): Promise<RepositorySnapshot> {
  const branchStatus = await api.getBranchStatus(repoPath);

  let commits: CommitListItem[] = [];
  let nextSkip = 0;
  let hasMore = false;
  let expectedSafeUnpushedCount = 0;

  do {
    const page = await api.getCommitHistory(repoPath, nextSkip, pageSize);
    commits = [...commits, ...page.items];
    nextSkip = page.nextSkip;
    hasMore = page.hasMore;
    expectedSafeUnpushedCount = page.safeUnpushedCount;
  } while (
    // 这里仍然只要求把全部安全 step-push 目标预加载出来；
    // 历史现在按全量展示，但 step push / push to commit 仍只能作用于 first-parent 安全路径。
    expectedSafeUnpushedCount > 0 &&
    commits.filter((commit) => commit.isSafePushTarget).length <
      expectedSafeUnpushedCount &&
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

  const details = await fetchCommitDetails(
    api,
    repoPath,
    selectedCommit.hash,
    ignoreWhitespace,
  );

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
