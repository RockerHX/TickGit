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
  fetchCommitDetails,
  fetchRepositoryIndex,
  fetchRepositorySnapshot,
  type TickGitPageApi,
} from "$lib/tickgit/page-data";

function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  const promise = new Promise<T>((innerResolve) => {
    resolve = innerResolve;
  });

  return { promise, resolve };
}

function repository(path: string): RepositorySummary {
  return {
    name: path.split("/").at(-1) ?? path,
    path,
    lastOpenedAt: 1,
  };
}

function branchStatus(overrides: Partial<BranchStatus> = {}): BranchStatus {
  return {
    branch: "main",
    upstream: "origin/main",
    aheadCount: 0,
    behindCount: 0,
    detached: false,
    pushAvailable: true,
    disabledReason: null,
    ...overrides,
  };
}

function commit(hash: string, isPushed = false): CommitListItem {
  return {
    hash,
    shortHash: hash.slice(0, 7),
    summary: hash,
    authorName: "TickGit",
    authorEmail: "tickgit@example.com",
    committedAt: "2026-04-25T12:00:00Z",
    tags: [],
    parents: [],
    isPushed,
  };
}

function commitMeta(overrides: Partial<CommitMeta> = {}): CommitMeta {
  return {
    body: "details",
    additions: 3,
    deletions: 1,
    ...overrides,
  };
}

function fileChange(
  path: string,
  overrides: Partial<CommitFileChange> = {},
): CommitFileChange {
  return {
    status: "M",
    path,
    previousPath: null,
    displayPath: path,
    ...overrides,
  };
}

function historyPage(
  items: CommitListItem[],
  overrides: Partial<CommitHistoryPage> = {},
): CommitHistoryPage {
  return {
    items,
    nextSkip: items.length,
    hasMore: false,
    unpushedCount: items.filter((item) => !item.isPushed).length,
    ...overrides,
  };
}

function createApiMock(
  overrides: Partial<TickGitPageApi> = {},
): TickGitPageApi {
  return {
    listRepositories: vi.fn().mockResolvedValue([]),
    getCurrentRepository: vi.fn().mockResolvedValue(null),
    getBranchStatus: vi.fn().mockResolvedValue(branchStatus()),
    getCommitHistory: vi.fn().mockResolvedValue(historyPage([])),
    getCommitFiles: vi.fn().mockResolvedValue([]),
    getCommitMeta: vi.fn().mockResolvedValue(commitMeta()),
    getCommitFileDiff: vi.fn().mockResolvedValue(""),
    ...overrides,
  };
}

describe("page data", () => {
  it("fetches repository index in parallel and returns both values", async () => {
    const repositoriesDeferred = deferred<RepositorySummary[]>();
    const currentRepositoryDeferred = deferred<RepositorySummary | null>();

    const listRepositories = vi.fn(() => repositoriesDeferred.promise);
    const getCurrentRepository = vi.fn(() => currentRepositoryDeferred.promise);

    const pending = fetchRepositoryIndex(
      createApiMock({
        listRepositories,
        getCurrentRepository,
      }),
    );

    expect(listRepositories).toHaveBeenCalledTimes(1);
    expect(getCurrentRepository).toHaveBeenCalledTimes(1);

    repositoriesDeferred.resolve([repository("/repo-a")]);
    currentRepositoryDeferred.resolve(repository("/repo-a"));

    await expect(pending).resolves.toEqual({
      repositories: [repository("/repo-a")],
      currentRepository: repository("/repo-a"),
    });
  });

  it("loads commit details and defaults to the first file diff", async () => {
    const getCommitFiles = vi
      .fn()
      .mockResolvedValue([fileChange("src/main.ts"), fileChange("src/app.ts")]);
    const getCommitMeta = vi.fn().mockResolvedValue(commitMeta());
    const getCommitFileDiff = vi.fn().mockResolvedValue("@@ diff");

    const details = await fetchCommitDetails(
      createApiMock({
        getCommitFiles,
        getCommitMeta,
        getCommitFileDiff,
      }),
      "/repo",
      "c1",
    );

    expect(details).toEqual({
      commitFiles: [fileChange("src/main.ts"), fileChange("src/app.ts")],
      commitMeta: commitMeta(),
      selectedFilePath: "src/main.ts",
      diffText: "@@ diff",
    });
    expect(getCommitFileDiff).toHaveBeenCalledWith(
      "/repo",
      "c1",
      "src/main.ts",
      false,
    );
  });

  it("skips diff loading when the selected commit has no files", async () => {
    const getCommitFileDiff = vi.fn();

    const details = await fetchCommitDetails(
      createApiMock({
        getCommitFiles: vi.fn().mockResolvedValue([]),
        getCommitMeta: vi.fn().mockResolvedValue(commitMeta()),
        getCommitFileDiff,
      }),
      "/repo",
      "c1",
    );

    expect(details).toEqual({
      commitFiles: [],
      commitMeta: commitMeta(),
      selectedFilePath: null,
      diffText: "",
    });
    expect(getCommitFileDiff).not.toHaveBeenCalled();
  });

  it("loads a repository snapshot with the selected commit details", async () => {
    const commits = [commit("c3"), commit("c2"), commit("c1")];
    const getCommitFiles = vi
      .fn()
      .mockResolvedValue([fileChange("src/main.ts")]);
    const getCommitMeta = vi.fn().mockResolvedValue(commitMeta());
    const getCommitFileDiff = vi.fn().mockResolvedValue("@@ diff");

    const snapshot = await fetchRepositorySnapshot(
      createApiMock({
        getBranchStatus: vi
          .fn()
          .mockResolvedValue(branchStatus({ aheadCount: 0 })),
        getCommitHistory: vi.fn().mockResolvedValue(
          historyPage(commits, {
            nextSkip: 3,
            hasMore: true,
          }),
        ),
        getCommitFiles,
        getCommitMeta,
        getCommitFileDiff,
      }),
      "/repo",
      50,
      false,
      null,
    );

    expect(snapshot.selectedCommit?.hash).toBe("c3");
    expect(snapshot.commitFiles).toEqual([fileChange("src/main.ts")]);
    expect(snapshot.commitMeta).toEqual(commitMeta());
    expect(snapshot.selectedFilePath).toBe("src/main.ts");
    expect(snapshot.diffText).toBe("@@ diff");
    expect(getCommitFiles).toHaveBeenCalledWith("/repo", "c3");
    expect(getCommitMeta).toHaveBeenCalledWith("/repo", "c3");
    expect(getCommitFileDiff).toHaveBeenCalledWith(
      "/repo",
      "c3",
      "src/main.ts",
      false,
    );
  });

  it("passes ignoreWhitespace through commit detail loading", async () => {
    const getCommitFileDiff = vi.fn().mockResolvedValue("@@ diff");

    await fetchCommitDetails(
      createApiMock({
        getCommitFiles: vi.fn().mockResolvedValue([fileChange("src/main.ts")]),
        getCommitMeta: vi.fn().mockResolvedValue(commitMeta()),
        getCommitFileDiff,
      }),
      "/repo",
      "c1",
      true,
    );

    expect(getCommitFileDiff).toHaveBeenCalledWith(
      "/repo",
      "c1",
      "src/main.ts",
      true,
    );
  });

  it("passes ignoreWhitespace through snapshot loading", async () => {
    const getCommitFileDiff = vi.fn().mockResolvedValue("@@ diff");

    await fetchRepositorySnapshot(
      createApiMock({
        getCommitHistory: vi
          .fn()
          .mockResolvedValue(
            historyPage([commit("c3"), commit("c2"), commit("c1")]),
          ),
        getCommitFiles: vi.fn().mockResolvedValue([fileChange("src/main.ts")]),
        getCommitMeta: vi.fn().mockResolvedValue(commitMeta()),
        getCommitFileDiff,
      }),
      "/repo",
      50,
      false,
      null,
      true,
    );

    expect(getCommitFileDiff).toHaveBeenCalledWith(
      "/repo",
      "c3",
      "src/main.ts",
      true,
    );
  });

  it("keeps the previous selection when it is still present", async () => {
    const snapshot = await fetchRepositorySnapshot(
      createApiMock({
        getCommitHistory: vi
          .fn()
          .mockResolvedValue(
            historyPage([commit("c3"), commit("c2"), commit("c1")]),
          ),
        getCommitFiles: vi
          .fn()
          .mockResolvedValue([fileChange("src/feature.ts")]),
      }),
      "/repo",
      50,
      true,
      "c2",
    );

    expect(snapshot.selectedCommit?.hash).toBe("c2");
  });

  it("falls back to the first commit when the previous selection disappeared", async () => {
    const snapshot = await fetchRepositorySnapshot(
      createApiMock({
        getCommitHistory: vi
          .fn()
          .mockResolvedValue(historyPage([commit("c3"), commit("c2")])),
        getCommitFiles: vi.fn().mockResolvedValue([fileChange("src/main.ts")]),
      }),
      "/repo",
      50,
      true,
      "missing",
    );

    expect(snapshot.selectedCommit?.hash).toBe("c3");
  });

  it("returns empty detail state when the repository has no commits", async () => {
    const getCommitFiles = vi.fn();
    const getCommitMeta = vi.fn();
    const getCommitFileDiff = vi.fn();

    const snapshot = await fetchRepositorySnapshot(
      createApiMock({
        getCommitHistory: vi.fn().mockResolvedValue(historyPage([])),
        getCommitFiles,
        getCommitMeta,
        getCommitFileDiff,
      }),
      "/repo",
      50,
      false,
      null,
    );

    expect(snapshot).toEqual({
      branchStatus: branchStatus(),
      commits: [],
      nextSkip: 0,
      hasMore: false,
      selectedCommit: null,
      commitMeta: null,
      commitFiles: [],
      selectedFilePath: null,
      diffText: "",
    });
    expect(getCommitFiles).not.toHaveBeenCalled();
    expect(getCommitMeta).not.toHaveBeenCalled();
    expect(getCommitFileDiff).not.toHaveBeenCalled();
  });

  it("loads extra history pages until all unpushed commits are available", async () => {
    const getCommitHistory = vi
      .fn()
      .mockResolvedValueOnce(
        historyPage([commit("c5"), commit("c4")], {
          nextSkip: 2,
          hasMore: true,
          unpushedCount: 4,
        }),
      )
      .mockResolvedValueOnce(
        historyPage([commit("c3"), commit("c2")], {
          nextSkip: 4,
          hasMore: true,
          unpushedCount: 4,
        }),
      );

    const snapshot = await fetchRepositorySnapshot(
      createApiMock({
        getBranchStatus: vi
          .fn()
          .mockResolvedValue(branchStatus({ aheadCount: 4 })),
        getCommitHistory,
        getCommitFiles: vi.fn().mockResolvedValue([fileChange("src/main.ts")]),
      }),
      "/repo",
      2,
      false,
      null,
    );

    expect(getCommitHistory).toHaveBeenNthCalledWith(1, "/repo", 0, 2);
    expect(getCommitHistory).toHaveBeenNthCalledWith(2, "/repo", 2, 2);
    expect(snapshot.commits.map((item) => item.hash)).toEqual([
      "c5",
      "c4",
      "c3",
      "c2",
    ]);
  });

  it("stops paging when the first page already covers all unpushed commits", async () => {
    const getCommitHistory = vi.fn().mockResolvedValue(
      historyPage([commit("c3"), commit("c2"), commit("c1", true)], {
        nextSkip: 3,
        hasMore: true,
      }),
    );

    await fetchRepositorySnapshot(
      createApiMock({
        getBranchStatus: vi
          .fn()
          .mockResolvedValue(branchStatus({ aheadCount: 2 })),
        getCommitHistory,
        getCommitFiles: vi.fn().mockResolvedValue([fileChange("src/main.ts")]),
      }),
      "/repo",
      50,
      false,
      null,
    );

    expect(getCommitHistory).toHaveBeenCalledTimes(1);
  });

  it("stops paging when there are no unpushed commits", async () => {
    const getCommitHistory = vi.fn().mockResolvedValue(
      historyPage([commit("c2", true), commit("c1", true)], {
        nextSkip: 2,
        hasMore: true,
      }),
    );

    await fetchRepositorySnapshot(
      createApiMock({
        getBranchStatus: vi
          .fn()
          .mockResolvedValue(branchStatus({ aheadCount: 0 })),
        getCommitHistory,
        getCommitFiles: vi.fn().mockResolvedValue([fileChange("src/main.ts")]),
      }),
      "/repo",
      50,
      false,
      null,
    );

    expect(getCommitHistory).toHaveBeenCalledTimes(1);
  });
});
