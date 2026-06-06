import type {
  BranchStatus,
  CommitHistoryFilters,
  CommitMeta,
  CommitFileChange,
  CommitFileDiffResult,
  CommitHistoryPage,
  CommitListItem,
  RepositorySummary,
} from "$lib/types";
import { pickCommitFileForPathFilter } from "$lib/tickgit/history";
import { pickSelectedCommit } from "$lib/tickgit/page-helpers";

export const EMPTY_DIFF_RESULT: CommitFileDiffResult = {
  text: "",
  isBinary: false,
  isImage: false,
  isTooLarge: false,
  truncated: false,
  byteCount: 0,
  lineCount: 0,
  oldImageDataUrl: null,
  newImageDataUrl: null,
};

export type CommitHistoryLoadOptions = {
  filters?: CommitHistoryFilters | null;
  preferredFilePathFilter?: string | null;
  skip?: number;
};

export type TickGitPageApi = {
  listRepositories: () => Promise<RepositorySummary[]>;
  getCurrentRepository: () => Promise<RepositorySummary | null>;
  getBranchStatus: (repoPath: string) => Promise<BranchStatus>;
  getCommitHistory: (
    repoPath: string,
    skip: number,
    limit: number,
    filters?: CommitHistoryFilters | null,
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
  ) => Promise<CommitFileDiffResult>;
};

export type RepositorySnapshot = {
  branchStatus: BranchStatus;
  commits: CommitListItem[];
  nextSkip: number;
  hasMore: boolean;
  totalCount: number;
  selectedCommit: CommitListItem | null;
  commitMeta: CommitMeta | null;
  commitFiles: CommitFileChange[];
  selectedFilePath: string | null;
  diffResult: CommitFileDiffResult;
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
  preferredFilePathFilter?: string | null,
) {
  const [commitFiles, commitMeta] = await Promise.all([
    api.getCommitFiles(repoPath, hash),
    api.getCommitMeta(repoPath, hash),
  ]);
  const selectedFile = pickCommitFileForPathFilter(
    commitFiles,
    preferredFilePathFilter,
  );
  const selectedFilePath = selectedFile?.path ?? null;
  // 没有文件变更时无需再请求 diff；否则既浪费一次 invoke，也会让空详情路径变得不明确。
  const diffResult = selectedFile
    ? await api.getCommitFileDiff(
        repoPath,
        hash,
        selectedFile.path,
        ignoreWhitespace,
        selectedFile.previousPath,
      )
    : EMPTY_DIFF_RESULT;

  return {
    commitFiles,
    commitMeta,
    selectedFilePath,
    diffResult,
  };
}

export async function fetchRepositorySnapshot(
  api: TickGitPageApi,
  repoPath: string,
  pageSize: number,
  keepSelection: boolean,
  previousSelectedHash: string | null,
  ignoreWhitespace = false,
  options: CommitHistoryLoadOptions = {},
): Promise<RepositorySnapshot> {
  const branchStatus = await api.getBranchStatus(repoPath);

  let commits: CommitListItem[] = [];
  const skip = options.skip ?? 0;
  let nextSkip = skip;
  let hasMore = false;
  let totalCount = 0;

  const page = await api.getCommitHistory(
    repoPath,
    skip,
    pageSize,
    options.filters,
  );
  commits = page.items;
  nextSkip = page.nextSkip;
  hasMore = page.hasMore;
  totalCount = page.totalCount;

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
      totalCount,
      selectedCommit: null,
      commitMeta: null,
      commitFiles: [],
      selectedFilePath: null,
      diffResult: EMPTY_DIFF_RESULT,
    };
  }

  const details = await fetchCommitDetails(
    api,
    repoPath,
    selectedCommit.hash,
    ignoreWhitespace,
    options.preferredFilePathFilter,
  );

  return {
    branchStatus,
    commits,
    nextSkip,
    hasMore,
    totalCount,
    selectedCommit,
    commitMeta: details.commitMeta,
    commitFiles: details.commitFiles,
    selectedFilePath: details.selectedFilePath,
    diffResult: details.diffResult,
  };
}
