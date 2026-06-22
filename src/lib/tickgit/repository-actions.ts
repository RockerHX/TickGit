import type { CommitHistoryFilters, RepositorySummary } from "$lib/types";
import {
  fetchCachedRepositoryOverview,
  fetchCommitDetailsOnly,
  fetchRepositoryIndexFast,
  fetchRepositoryOverviewSnapshot,
  fetchRepositoryIndex,
  fetchRepositorySnapshot,
  refreshRepositoryStatuses,
  snapshotFromOverview,
  type RepositorySnapshot,
  type TickGitPageApi,
  type TickGitOptimizedPageApi,
} from "$lib/tickgit/page-data";
import { isRepositoryAvailable } from "$lib/tickgit/page-state";
import type { CommitListItem } from "$lib/types";

export type RepositoryActionsApi = TickGitPageApi & {
  refreshRemoteTracking: (repoPath: string) => Promise<void>;
  listLocalBranches: (repoPath: string) => Promise<string[]>;
};

export type OptimizedRepositoryActionsApi = RepositoryActionsApi &
  TickGitOptimizedPageApi;

export type LoadRepositoryStateOptions = {
  pageSize: number;
  historySkip?: number;
  keepSelection: boolean;
  previousSelectedHash: string | null;
  ignoreWhitespace: boolean;
  refreshRemoteTracking?: boolean;
  filters?: CommitHistoryFilters | null;
  preferredFilePathFilter?: string | null;
};

export type RepositoryStateResult = {
  snapshot: RepositorySnapshot;
  branches: string[];
  remoteRefreshError: unknown | null;
};

export type BootstrapRepositoryStateResult = {
  repositories: RepositorySummary[];
  currentRepository: RepositorySummary | null;
  repositoryState: RepositoryStateResult | null;
};

export type OptimizedBootstrapRepositoryStateResult = {
  repositories: RepositorySummary[];
  currentRepository: RepositorySummary | null;
  repositoryState: RepositoryStateResult | null;
  cacheApplied: boolean;
};

export async function loadRepositoryIndex(api: TickGitPageApi) {
  return fetchRepositoryIndex(api);
}

export async function loadRepositoryIndexFast(
  api: OptimizedRepositoryActionsApi,
) {
  return fetchRepositoryIndexFast(api);
}

export async function refreshRepositoryAvailability(
  api: OptimizedRepositoryActionsApi,
  repositories: RepositorySummary[],
) {
  return refreshRepositoryStatuses(api, repositories);
}

export async function loadRepositoryOverviewState(
  api: OptimizedRepositoryActionsApi,
  repoPath: string,
  options: LoadRepositoryStateOptions,
): Promise<RepositoryStateResult> {
  const snapshot = await fetchRepositoryOverviewSnapshot(
    api,
    repoPath,
    options.pageSize,
    options.keepSelection,
    options.previousSelectedHash,
    {
      filters: options.filters,
      preferredFilePathFilter: options.preferredFilePathFilter,
      skip: options.historySkip,
    },
  );

  return {
    snapshot,
    branches: snapshot.branches,
    remoteRefreshError: null,
  };
}

export async function loadCommitDetailsState(
  api: OptimizedRepositoryActionsApi,
  repoPath: string,
  commit: CommitListItem,
  preferredFilePathFilter?: string | null,
) {
  return fetchCommitDetailsOnly(
    api,
    repoPath,
    commit.hash,
    preferredFilePathFilter,
  );
}

export async function loadRepositoryStateSnapshot(
  api: RepositoryActionsApi,
  repoPath: string,
  options: LoadRepositoryStateOptions,
): Promise<RepositoryStateResult> {
  let remoteRefreshError: unknown | null = null;

  if (options.refreshRemoteTracking) {
    try {
      await api.refreshRemoteTracking(repoPath);
    } catch (error) {
      remoteRefreshError = error;
    }
  }

  const [snapshot, branches] = await Promise.all([
    fetchRepositorySnapshot(
      api,
      repoPath,
      options.pageSize,
      options.keepSelection,
      options.previousSelectedHash,
      options.ignoreWhitespace,
      {
        filters: options.filters,
        preferredFilePathFilter: options.preferredFilePathFilter,
        skip: options.historySkip,
      },
    ),
    api.listLocalBranches(repoPath),
  ]);

  return {
    snapshot,
    branches,
    remoteRefreshError,
  };
}

export async function loadBootstrapRepositoryState(
  api: RepositoryActionsApi,
  options: LoadRepositoryStateOptions,
): Promise<BootstrapRepositoryStateResult> {
  const repositoryIndex = await fetchRepositoryIndex(api);
  const currentRepository = repositoryIndex.currentRepository;

  if (!currentRepository || !isRepositoryAvailable(currentRepository)) {
    return {
      ...repositoryIndex,
      repositoryState: null,
    };
  }

  const repositoryState = await loadRepositoryStateSnapshot(
    api,
    currentRepository.path,
    options,
  );

  return {
    ...repositoryIndex,
    repositoryState,
  };
}

export async function loadOptimizedBootstrapRepositoryState(
  api: OptimizedRepositoryActionsApi,
  options: LoadRepositoryStateOptions,
): Promise<OptimizedBootstrapRepositoryStateResult> {
  const repositoryIndex = await fetchRepositoryIndexFast(api);
  const currentRepository = repositoryIndex.currentRepository;
  let repositoryState: RepositoryStateResult | null = null;
  let cacheApplied = false;

  if (currentRepository && isRepositoryAvailable(currentRepository)) {
    const cached = await fetchCachedRepositoryOverview(api).catch(() => null);
    if (
      cached &&
      cached.repoPath === currentRepository.path &&
      cached.skip === (options.historySkip ?? 0) &&
      cached.limit === options.pageSize
    ) {
      const snapshot = snapshotFromOverview(
        cached.overview,
        options.keepSelection,
        options.previousSelectedHash,
      );
      repositoryState = {
        snapshot,
        branches: snapshot.branches,
        remoteRefreshError: null,
      };
      cacheApplied = true;
    }
  }

  return {
    ...repositoryIndex,
    repositoryState,
    cacheApplied,
  };
}
