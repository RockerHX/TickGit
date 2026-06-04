import type { RepositorySummary } from "$lib/types";
import {
  fetchRepositoryIndex,
  fetchRepositorySnapshot,
  type RepositorySnapshot,
  type TickGitPageApi,
} from "$lib/tickgit/page-data";
import { isRepositoryAvailable } from "$lib/tickgit/page-state";

export type RepositoryActionsApi = TickGitPageApi & {
  refreshRemoteTracking: (repoPath: string) => Promise<void>;
  listLocalBranches: (repoPath: string) => Promise<string[]>;
};

export type LoadRepositoryStateOptions = {
  pageSize: number;
  keepSelection: boolean;
  previousSelectedHash: string | null;
  ignoreWhitespace: boolean;
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

export async function loadRepositoryIndex(api: TickGitPageApi) {
  return fetchRepositoryIndex(api);
}

export async function loadRepositoryStateSnapshot(
  api: RepositoryActionsApi,
  repoPath: string,
  options: LoadRepositoryStateOptions,
): Promise<RepositoryStateResult> {
  let remoteRefreshError: unknown | null = null;

  try {
    await api.refreshRemoteTracking(repoPath);
  } catch (error) {
    remoteRefreshError = error;
  }

  const [snapshot, branches] = await Promise.all([
    fetchRepositorySnapshot(
      api,
      repoPath,
      options.pageSize,
      options.keepSelection,
      options.previousSelectedHash,
      options.ignoreWhitespace,
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
