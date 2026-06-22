import type {
  BranchStatus,
  CommitHistoryFilters,
  CommitMeta,
  CommitFileChange,
  CommitFileDiffResult,
  CommitHistoryPage,
  CommitListItem,
  CommitDetails,
  RepositoryIndex,
  RepositoryOverview,
  RepositoryOverviewCacheEntry,
  RepositoryStatusUpdate,
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

export type TickGitOptimizedPageApi = TickGitPageApi & {
  getRepositoryIndexFast: () => Promise<RepositoryIndex>;
  getCachedRepositoryOverview: () => Promise<RepositoryOverviewCacheEntry | null>;
  refreshRepositoryStatuses: (
    paths: string[],
  ) => Promise<RepositoryStatusUpdate[]>;
  getRepositoryOverview: (
    repoPath: string,
    skip: number,
    limit: number,
    filters?: CommitHistoryFilters | null,
  ) => Promise<RepositoryOverview>;
  getCommitDetails: (repoPath: string, hash: string) => Promise<CommitDetails>;
};

export type RepositorySnapshot = {
  branchStatus: BranchStatus;
  branches: string[];
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

export async function fetchRepositoryIndexFast(api: TickGitOptimizedPageApi) {
  return api.getRepositoryIndexFast();
}

export async function fetchCachedRepositoryOverview(
  api: TickGitOptimizedPageApi,
) {
  return api.getCachedRepositoryOverview();
}

export async function refreshRepositoryStatuses(
  api: TickGitOptimizedPageApi,
  repositories: RepositorySummary[],
) {
  const updates = await api.refreshRepositoryStatuses(
    repositories.map((repository) => repository.path),
  );
  const updatesByPath = new Map(
    updates.map((update) => [update.path, update] as const),
  );

  return repositories.map((repository) => {
    const update = updatesByPath.get(repository.path);
    return update
      ? {
          ...repository,
          status: update.status,
          disabledReason: update.disabledReason,
          disabledReasonCode: update.disabledReasonCode,
        }
      : repository;
  });
}

export async function fetchRepositoryOverviewSnapshot(
  api: TickGitOptimizedPageApi,
  repoPath: string,
  pageSize: number,
  keepSelection: boolean,
  previousSelectedHash: string | null,
  options: CommitHistoryLoadOptions = {},
): Promise<RepositorySnapshot> {
  const overview = await api.getRepositoryOverview(
    repoPath,
    options.skip ?? 0,
    pageSize,
    options.filters,
  );
  return snapshotFromOverview(overview, keepSelection, previousSelectedHash);
}

export function snapshotFromOverview(
  overview: RepositoryOverview,
  keepSelection: boolean,
  previousSelectedHash: string | null,
): RepositorySnapshot {
  const commits = overview.history.items;
  const selectedCommit = pickSelectedCommit(
    commits,
    previousSelectedHash,
    keepSelection,
  );

  return {
    branchStatus: overview.branchStatus,
    branches: overview.branches,
    commits,
    nextSkip: overview.history.nextSkip,
    hasMore: overview.history.hasMore,
    totalCount: overview.history.totalCount,
    selectedCommit,
    commitMeta: null,
    commitFiles: [],
    selectedFilePath: null,
    diffResult: EMPTY_DIFF_RESULT,
  };
}

export async function fetchCommitDetailsOnly(
  api: Pick<TickGitOptimizedPageApi, "getCommitDetails">,
  repoPath: string,
  hash: string,
  preferredFilePathFilter?: string | null,
) {
  const details = await api.getCommitDetails(repoPath, hash);
  const selectedFile = pickCommitFileForPathFilter(
    details.files,
    preferredFilePathFilter,
  );

  return {
    commitFiles: details.files,
    commitMeta: details.meta,
    selectedFilePath: selectedFile?.path ?? null,
  };
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
      branches: [],
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
    branches: [],
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
