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
    status: "available",
    disabledReason: null,
  };
}

function branchStatus(overrides: Partial<BranchStatus> = {}): BranchStatus {
  return {
    branch: "main",
    upstream: "origin/main",
    aheadCount: 0,
    safeAheadCount: 0,
    behindCount: 0,
    detached: false,
    pushAvailable: true,
    disabledReason: null,
    ...overrides,
  };
}

function commit(
  hash: string,
  isPushed = false,
  isSafePushTarget = !isPushed,
): CommitListItem {
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
    isSafePushTarget,
    pushBlockedReason:
      !isPushed && !isSafePushTarget
        ? "该 Commit 未推送，但不在 first-parent 安全路径上，不能作为 step push / push to commit 目标"
        : null,
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
    totalCount: items.length,
    unpushedCount: items.filter((item) => !item.isPushed).length,
    safeUnpushedCount: items.filter((item) => item.isSafePushTarget).length,
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
    getCommitFileDiff: vi.fn().mockResolvedValue(diffResult()),
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
    const getCommitFileDiff = vi.fn().mockResolvedValue(diffResult("@@ diff"));

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
      diffResult: diffResult("@@ diff"),
    });
    expect(getCommitFileDiff).toHaveBeenCalledWith(
      "/repo",
      "c1",
      "src/main.ts",
      false,
      null,
    );
  });

  it("loads commit details with the first file matching a path filter", async () => {
    const getCommitFileDiff = vi.fn().mockResolvedValue(diffResult("@@ diff"));

    const details = await fetchCommitDetails(
      createApiMock({
        getCommitFiles: vi
          .fn()
          .mockResolvedValue([
            fileChange("README.md"),
            fileChange("src/main.ts"),
            fileChange("src/app.ts"),
          ]),
        getCommitMeta: vi.fn().mockResolvedValue(commitMeta()),
        getCommitFileDiff,
      }),
      "/repo",
      "c1",
      false,
      "src/app",
    );

    expect(details.selectedFilePath).toBe("src/app.ts");
    expect(getCommitFileDiff).toHaveBeenCalledWith(
      "/repo",
      "c1",
      "src/app.ts",
      false,
      null,
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
      diffResult: diffResult(),
    });
    expect(getCommitFileDiff).not.toHaveBeenCalled();
  });

  it("loads a repository snapshot with the selected commit details", async () => {
    const commits = [commit("c3"), commit("c2"), commit("c1")];
    const getCommitFiles = vi
      .fn()
      .mockResolvedValue([fileChange("src/main.ts")]);
    const getCommitMeta = vi.fn().mockResolvedValue(commitMeta());
    const getCommitFileDiff = vi.fn().mockResolvedValue(diffResult("@@ diff"));

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
    expect(snapshot.diffResult).toEqual(diffResult("@@ diff"));
    expect(getCommitFiles).toHaveBeenCalledWith("/repo", "c3");
    expect(getCommitMeta).toHaveBeenCalledWith("/repo", "c3");
    expect(getCommitFileDiff).toHaveBeenCalledWith(
      "/repo",
      "c3",
      "src/main.ts",
      false,
      null,
    );
  });

  it("passes ignoreWhitespace through commit detail loading", async () => {
    const getCommitFileDiff = vi.fn().mockResolvedValue(diffResult("@@ diff"));

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
      null,
    );
  });

  it("passes previousPath through renamed file diff loading", async () => {
    const getCommitFileDiff = vi.fn().mockResolvedValue(diffResult("@@ diff"));

    await fetchCommitDetails(
      createApiMock({
        getCommitFiles: vi.fn().mockResolvedValue([
          fileChange("src/new.ts", {
            status: "R100",
            previousPath: "src/old.ts",
            displayPath: "src/old.ts -> src/new.ts",
          }),
        ]),
        getCommitMeta: vi.fn().mockResolvedValue(commitMeta()),
        getCommitFileDiff,
      }),
      "/repo",
      "c1",
    );

    expect(getCommitFileDiff).toHaveBeenCalledWith(
      "/repo",
      "c1",
      "src/new.ts",
      false,
      "src/old.ts",
    );
  });

  it("passes ignoreWhitespace through snapshot loading", async () => {
    const getCommitFileDiff = vi.fn().mockResolvedValue(diffResult("@@ diff"));

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
      null,
    );
  });

  it("passes history filters and preferred file filter through snapshot loading", async () => {
    const getCommitHistory = vi
      .fn()
      .mockResolvedValue(historyPage([commit("c3")]));
    const getCommitFileDiff = vi.fn().mockResolvedValue(diffResult("@@ diff"));

    const snapshot = await fetchRepositorySnapshot(
      createApiMock({
        getCommitHistory,
        getCommitFiles: vi
          .fn()
          .mockResolvedValue([
            fileChange("README.md"),
            fileChange("src/filter-match.ts"),
          ]),
        getCommitMeta: vi.fn().mockResolvedValue(commitMeta()),
        getCommitFileDiff,
      }),
      "/repo",
      50,
      false,
      null,
      false,
      {
        filters: { query: "fix", author: "Ada", filePath: "src/filter" },
        preferredFilePathFilter: "src/filter",
      },
    );

    expect(getCommitHistory).toHaveBeenCalledWith("/repo", 0, 50, {
      query: "fix",
      author: "Ada",
      filePath: "src/filter",
    });
    expect(snapshot.selectedFilePath).toBe("src/filter-match.ts");
    expect(getCommitFileDiff).toHaveBeenCalledWith(
      "/repo",
      "c3",
      "src/filter-match.ts",
      false,
      null,
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
      branches: [],
      commits: [],
      nextSkip: 0,
      hasMore: false,
      totalCount: 0,
      selectedCommit: null,
      commitMeta: null,
      commitFiles: [],
      selectedFilePath: null,
      diffResult: diffResult(),
    });
    expect(getCommitFiles).not.toHaveBeenCalled();
    expect(getCommitMeta).not.toHaveBeenCalled();
    expect(getCommitFileDiff).not.toHaveBeenCalled();
  });

  it("loads only the requested history page for repository snapshots", async () => {
    const getCommitHistory = vi.fn().mockResolvedValue(
      historyPage([commit("c3"), commit("c2")], {
        nextSkip: 4,
        hasMore: true,
        totalCount: 8,
        unpushedCount: 4,
        safeUnpushedCount: 4,
      }),
    );

    const snapshot = await fetchRepositorySnapshot(
      createApiMock({
        getBranchStatus: vi
          .fn()
          .mockResolvedValue(
            branchStatus({ aheadCount: 4, safeAheadCount: 4 }),
          ),
        getCommitHistory,
        getCommitFiles: vi.fn().mockResolvedValue([fileChange("src/main.ts")]),
      }),
      "/repo",
      2,
      false,
      null,
      false,
      { skip: 2 },
    );

    expect(getCommitHistory).toHaveBeenCalledTimes(1);
    expect(getCommitHistory).toHaveBeenCalledWith("/repo", 2, 2, undefined);
    expect(snapshot.commits.map((item) => item.hash)).toEqual(["c3", "c2"]);
    expect(snapshot.nextSkip).toBe(4);
    expect(snapshot.hasMore).toBe(true);
    expect(snapshot.totalCount).toBe(8);
  });

  it("keeps only the current page even when more safe commits exist", async () => {
    const getCommitHistory = vi.fn().mockResolvedValue(
      historyPage([commit("pushed", true), commit("c2")], {
        nextSkip: 2,
        hasMore: true,
        totalCount: 3,
        unpushedCount: 2,
        safeUnpushedCount: 2,
      }),
    );

    const snapshot = await fetchRepositorySnapshot(
      createApiMock({
        getBranchStatus: vi
          .fn()
          .mockResolvedValue(
            branchStatus({ aheadCount: 2, safeAheadCount: 2 }),
          ),
        getCommitHistory,
        getCommitFiles: vi.fn().mockResolvedValue([fileChange("src/main.ts")]),
      }),
      "/repo",
      2,
      false,
      null,
    );

    expect(getCommitHistory).toHaveBeenCalledTimes(1);
    expect(snapshot.commits.map((item) => item.hash)).toEqual(["pushed", "c2"]);
  });
});
