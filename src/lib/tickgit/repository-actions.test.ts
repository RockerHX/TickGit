import { describe, expect, it, vi } from "vitest";
import type {
  BranchStatus,
  CommitFileChange,
  CommitHistoryPage,
  CommitListItem,
  CommitMeta,
  RepositorySummary,
} from "$lib/types";
import {
  loadBootstrapRepositoryState,
  loadRepositoryIndex,
  loadRepositoryStateSnapshot,
  type RepositoryActionsApi,
} from "$lib/tickgit/repository-actions";

function repository(path: string): RepositorySummary {
  return {
    name: path.split("/").at(-1) ?? path,
    path,
    lastOpenedAt: 1,
  };
}

function branchStatus(): BranchStatus {
  return {
    branch: "main",
    upstream: "origin/main",
    aheadCount: 0,
    safeAheadCount: 0,
    behindCount: 0,
    detached: false,
    pushAvailable: true,
    disabledReason: null,
  };
}

function commit(hash: string): CommitListItem {
  return {
    hash,
    shortHash: hash.slice(0, 7),
    summary: hash,
    authorName: "TickGit",
    authorEmail: "tickgit@example.com",
    committedAt: "2026-04-25T12:00:00Z",
    tags: [],
    parents: [],
    isPushed: true,
    isSafePushTarget: false,
    pushBlockedReason: null,
  };
}

function commitMeta(): CommitMeta {
  return {
    body: "details",
    additions: 1,
    deletions: 0,
  };
}

function fileChange(path: string): CommitFileChange {
  return {
    status: "M",
    path,
    previousPath: null,
    displayPath: path,
  };
}

function historyPage(items: CommitListItem[]): CommitHistoryPage {
  return {
    items,
    nextSkip: items.length,
    hasMore: false,
    unpushedCount: 0,
    safeUnpushedCount: 0,
  };
}

function createApiMock(
  overrides: Partial<RepositoryActionsApi> = {},
): RepositoryActionsApi {
  const commits = [commit("c1")];

  return {
    listRepositories: vi.fn().mockResolvedValue([]),
    getCurrentRepository: vi.fn().mockResolvedValue(null),
    refreshRemoteTracking: vi.fn().mockResolvedValue(undefined),
    listLocalBranches: vi.fn().mockResolvedValue(["main"]),
    getBranchStatus: vi.fn().mockResolvedValue(branchStatus()),
    getCommitHistory: vi.fn().mockResolvedValue(historyPage(commits)),
    getCommitFiles: vi.fn().mockResolvedValue([fileChange("src/main.ts")]),
    getCommitMeta: vi.fn().mockResolvedValue(commitMeta()),
    getCommitFileDiff: vi.fn().mockResolvedValue("@@ diff"),
    ...overrides,
  };
}

describe("repository actions", () => {
  it("loads repository index through page data", async () => {
    const repositories = [repository("/repo-a")];
    const currentRepository = repository("/repo-a");
    const api = createApiMock({
      listRepositories: vi.fn().mockResolvedValue(repositories),
      getCurrentRepository: vi.fn().mockResolvedValue(currentRepository),
    });

    await expect(loadRepositoryIndex(api)).resolves.toEqual({
      repositories,
      currentRepository,
    });
  });

  it("loads bootstrap state with current repository snapshot", async () => {
    const repositories = [repository("/repo-a")];
    const currentRepository = repository("/repo-a");
    const api = createApiMock({
      listRepositories: vi.fn().mockResolvedValue(repositories),
      getCurrentRepository: vi.fn().mockResolvedValue(currentRepository),
      listLocalBranches: vi.fn().mockResolvedValue(["main", "feature"]),
    });

    const state = await loadBootstrapRepositoryState(api, {
      pageSize: 50,
      keepSelection: false,
      previousSelectedHash: null,
      ignoreWhitespace: false,
    });

    expect(state.repositories).toEqual(repositories);
    expect(state.currentRepository).toEqual(currentRepository);
    expect(state.repositoryState?.branches).toEqual(["main", "feature"]);
    expect(state.repositoryState?.snapshot.selectedCommit?.hash).toBe("c1");
  });

  it("keeps loading snapshot when remote refresh fails", async () => {
    const remoteRefreshError = new Error("fetch failed");
    const api = createApiMock({
      refreshRemoteTracking: vi.fn().mockRejectedValue(remoteRefreshError),
    });

    const state = await loadRepositoryStateSnapshot(api, "/repo-a", {
      pageSize: 50,
      keepSelection: true,
      previousSelectedHash: "c1",
      ignoreWhitespace: false,
    });

    expect(state.remoteRefreshError).toBe(remoteRefreshError);
    expect(state.branches).toEqual(["main"]);
    expect(state.snapshot.selectedCommit?.hash).toBe("c1");
    expect(api.getBranchStatus).toHaveBeenCalledWith("/repo-a");
  });
});
