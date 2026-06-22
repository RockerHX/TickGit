import { describe, expect, it, vi } from "vitest";
import type {
  BranchStatus,
  CommitFileChange,
  CommitFileDiffResult,
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
    status: "available",
    disabledReason: null,
  };
}

function missingRepository(path: string): RepositorySummary {
  return {
    ...repository(path),
    status: "missing",
    disabledReason: "仓库路径不存在",
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

function diffResult(text = ""): CommitFileDiffResult {
  return {
    text,
    isBinary: false,
    isImage: false,
    isTooLarge: false,
    truncated: false,
    byteCount: text.length,
    lineCount: text ? text.split("\n").length : 0,
    oldImageDataUrl: null,
    newImageDataUrl: null,
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
    totalCount: items.length,
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
    getRepositoryRevision: vi.fn().mockResolvedValue({
      head: "h1",
      branch: "main",
      upstream: null,
    }),
    getCommitHistory: vi.fn().mockResolvedValue(historyPage(commits)),
    getCommitFiles: vi.fn().mockResolvedValue([fileChange("src/main.ts")]),
    getCommitMeta: vi.fn().mockResolvedValue(commitMeta()),
    getCommitFileDiff: vi.fn().mockResolvedValue(diffResult("@@ diff")),
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
    const refreshRemoteTracking = vi.fn().mockResolvedValue(undefined);
    const api = createApiMock({
      listRepositories: vi.fn().mockResolvedValue(repositories),
      getCurrentRepository: vi.fn().mockResolvedValue(currentRepository),
      refreshRemoteTracking,
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
    expect(refreshRemoteTracking).not.toHaveBeenCalled();
  });

  it("does not load repository snapshot when current repository is unavailable", async () => {
    const repositories = [missingRepository("/repo-a")];
    const currentRepository = missingRepository("/repo-a");
    const getBranchStatus = vi.fn().mockResolvedValue(branchStatus());
    const api = createApiMock({
      listRepositories: vi.fn().mockResolvedValue(repositories),
      getCurrentRepository: vi.fn().mockResolvedValue(currentRepository),
      getBranchStatus,
    });

    const state = await loadBootstrapRepositoryState(api, {
      pageSize: 50,
      keepSelection: false,
      previousSelectedHash: null,
      ignoreWhitespace: false,
    });

    expect(state.repositories).toEqual(repositories);
    expect(state.currentRepository).toEqual(currentRepository);
    expect(state.repositoryState).toBeNull();
    expect(getBranchStatus).not.toHaveBeenCalled();
  });

  it("loads repository snapshot without automatically refreshing remote tracking", async () => {
    const refreshRemoteTracking = vi.fn().mockResolvedValue(undefined);
    const api = createApiMock({ refreshRemoteTracking });

    const state = await loadRepositoryStateSnapshot(api, "/repo-a", {
      pageSize: 50,
      keepSelection: true,
      previousSelectedHash: "c1",
      ignoreWhitespace: false,
    });

    expect(state.remoteRefreshError).toBeNull();
    expect(refreshRemoteTracking).not.toHaveBeenCalled();
    expect(state.branches).toEqual(["main"]);
    expect(state.snapshot.selectedCommit?.hash).toBe("c1");
    expect(api.getBranchStatus).toHaveBeenCalledWith("/repo-a");
  });

  it("passes cached commit details into snapshot loading", async () => {
    const cachedFiles = [fileChange("src/main.ts")];
    const getCommitFiles = vi.fn();
    const getCommitMeta = vi.fn();
    const getCommitFileDiff = vi.fn();
    const api = createApiMock({
      getCommitHistory: vi.fn().mockResolvedValue(historyPage([commit("c1")])),
      getCommitFiles,
      getCommitMeta,
      getCommitFileDiff,
    });

    const state = await loadRepositoryStateSnapshot(api, "/repo-a", {
      pageSize: 50,
      keepSelection: true,
      previousSelectedHash: "c1",
      ignoreWhitespace: false,
      preferredFilePathFilter: "src/main",
      cachedCommitDetails: {
        hash: "c1",
        ignoreWhitespace: false,
        preferredFilePathFilter: "src/main",
        commitFiles: cachedFiles,
        commitMeta: commitMeta(),
        selectedFilePath: "src/main.ts",
        diffResult: diffResult("@@ cached"),
      },
    });

    expect(state.snapshot.commitFiles).toBe(cachedFiles);
    expect(state.snapshot.diffResult).toEqual(diffResult("@@ cached"));
    expect(getCommitFiles).not.toHaveBeenCalled();
    expect(getCommitMeta).not.toHaveBeenCalled();
    expect(getCommitFileDiff).not.toHaveBeenCalled();
  });

  it("keeps loading snapshot when manual remote refresh fails", async () => {
    const remoteRefreshError = new Error("fetch failed");
    const api = createApiMock({
      refreshRemoteTracking: vi.fn().mockRejectedValue(remoteRefreshError),
    });

    const state = await loadRepositoryStateSnapshot(api, "/repo-a", {
      pageSize: 50,
      keepSelection: true,
      previousSelectedHash: "c1",
      ignoreWhitespace: false,
      refreshRemoteTracking: true,
    });

    expect(state.remoteRefreshError).toBe(remoteRefreshError);
    expect(api.refreshRemoteTracking).toHaveBeenCalledWith("/repo-a");
    expect(state.branches).toEqual(["main"]);
    expect(state.snapshot.selectedCommit?.hash).toBe("c1");
    expect(api.getBranchStatus).toHaveBeenCalledWith("/repo-a");
  });

  it("passes history filters and preferred file filters into snapshot loading", async () => {
    const getCommitHistory = vi
      .fn()
      .mockResolvedValue(historyPage([commit("c1")]));
    const getCommitFileDiff = vi.fn().mockResolvedValue(diffResult("@@ diff"));
    const api = createApiMock({
      getCommitHistory,
      getCommitFiles: vi
        .fn()
        .mockResolvedValue([
          fileChange("README.md"),
          fileChange("src/matched.ts"),
        ]),
      getCommitFileDiff,
    });

    const state = await loadRepositoryStateSnapshot(api, "/repo-a", {
      pageSize: 50,
      historySkip: 20,
      keepSelection: false,
      previousSelectedHash: null,
      ignoreWhitespace: false,
      filters: { query: "fix", author: "Ada", filePath: "src/matched" },
      preferredFilePathFilter: "src/matched",
    });

    expect(getCommitHistory).toHaveBeenCalledWith("/repo-a", 20, 50, {
      query: "fix",
      author: "Ada",
      filePath: "src/matched",
    });
    expect(state.snapshot.selectedFilePath).toBe("src/matched.ts");
    expect(getCommitFileDiff).toHaveBeenCalledWith(
      "/repo-a",
      "c1",
      "src/matched.ts",
      false,
      null,
    );
  });
});
