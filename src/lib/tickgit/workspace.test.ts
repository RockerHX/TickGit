import { describe, expect, it, vi } from "vitest";
import type {
  CommitFileDiffResult,
  WorkspaceChangeSection,
  WorkspaceFileChange,
  WorkspaceStatus,
} from "$lib/types";
import {
  EMPTY_WORKSPACE_STATUS,
  fetchWorkspaceSnapshot,
  getWorkspaceCommitFailureEffect,
  getWorkspaceCommitSuccessEffect,
  pickWorkspaceFile,
  type WorkspaceApi,
} from "$lib/tickgit/workspace";

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

function workspaceFile(
  section: WorkspaceChangeSection,
  path: string,
  status = section === "staged" ? "M" : "??",
): WorkspaceFileChange {
  return {
    section,
    kind: status === "??" ? "untracked" : "modified",
    status,
    path,
    previousPath: null,
    displayPath: path,
  };
}

function workspaceStatus(
  staged: WorkspaceFileChange[] = [],
  unstaged: WorkspaceFileChange[] = [],
): WorkspaceStatus {
  return { staged, unstaged };
}

function createApiMock(overrides: Partial<WorkspaceApi> = {}): WorkspaceApi {
  return {
    getWorkspaceStatus: vi.fn().mockResolvedValue(EMPTY_WORKSPACE_STATUS),
    getWorkspaceFileDiff: vi.fn().mockResolvedValue(diffResult("@@ diff")),
    ...overrides,
  };
}

describe("workspace data", () => {
  it("defaults to the first staged file before unstaged files", () => {
    const staged = workspaceFile("staged", "staged.txt");
    const unstaged = workspaceFile("unstaged", "unstaged.txt");

    expect(
      pickWorkspaceFile(workspaceStatus([staged], [unstaged]), null, false),
    ).toEqual(staged);
    expect(
      pickWorkspaceFile(workspaceStatus([], [unstaged]), null, false),
    ).toEqual(unstaged);
  });

  it("keeps the previous selection when it still exists", async () => {
    const staged = workspaceFile("staged", "staged.txt");
    const unstaged = workspaceFile("unstaged", "unstaged.txt");
    const getWorkspaceFileDiff = vi
      .fn()
      .mockResolvedValue(diffResult("@@ diff"));

    const snapshot = await fetchWorkspaceSnapshot(
      createApiMock({
        getWorkspaceStatus: vi
          .fn()
          .mockResolvedValue(workspaceStatus([staged], [unstaged])),
        getWorkspaceFileDiff,
      }),
      "/repo",
      true,
      { section: "unstaged", path: "unstaged.txt" },
    );

    expect(snapshot.selectedFile).toEqual(unstaged);
    expect(snapshot.selectedSection).toBe("unstaged");
    expect(snapshot.selectedFilePath).toBe("unstaged.txt");
    expect(getWorkspaceFileDiff).toHaveBeenCalledWith(
      "/repo",
      "unstaged",
      "unstaged.txt",
      false,
      null,
    );
  });

  it("falls back when the previous selection no longer exists", async () => {
    const staged = workspaceFile("staged", "staged.txt");

    const snapshot = await fetchWorkspaceSnapshot(
      createApiMock({
        getWorkspaceStatus: vi
          .fn()
          .mockResolvedValue(workspaceStatus([staged])),
      }),
      "/repo",
      true,
      { section: "unstaged", path: "missing.txt" },
    );

    expect(snapshot.selectedFile).toEqual(staged);
    expect(snapshot.selectedSection).toBe("staged");
    expect(snapshot.selectedFilePath).toBe("staged.txt");
  });

  it("falls back to the staged version after a file action changes section", async () => {
    const staged = workspaceFile("staged", "same.txt");

    const snapshot = await fetchWorkspaceSnapshot(
      createApiMock({
        getWorkspaceStatus: vi
          .fn()
          .mockResolvedValue(workspaceStatus([staged])),
      }),
      "/repo",
      true,
      { section: "unstaged", path: "same.txt" },
    );

    expect(snapshot.selectedFile).toEqual(staged);
    expect(snapshot.selectedSection).toBe("staged");
    expect(snapshot.selectedFilePath).toBe("same.txt");
  });

  it("skips diff loading when there are no workspace files", async () => {
    const getWorkspaceFileDiff = vi.fn();

    const snapshot = await fetchWorkspaceSnapshot(
      createApiMock({
        getWorkspaceStatus: vi.fn().mockResolvedValue(EMPTY_WORKSPACE_STATUS),
        getWorkspaceFileDiff,
      }),
      "/repo",
      false,
      null,
    );

    expect(snapshot).toEqual({
      status: EMPTY_WORKSPACE_STATUS,
      selectedFile: null,
      selectedSection: null,
      selectedFilePath: null,
      diffResult: diffResult(),
    });
    expect(getWorkspaceFileDiff).not.toHaveBeenCalled();
  });

  it("passes ignore whitespace and previous path through diff loading", async () => {
    const renamed = {
      ...workspaceFile("staged", "new.txt", "R"),
      previousPath: "old.txt",
      displayPath: "old.txt -> new.txt",
    };
    const getWorkspaceFileDiff = vi
      .fn()
      .mockResolvedValue(diffResult("@@ diff"));

    await fetchWorkspaceSnapshot(
      createApiMock({
        getWorkspaceStatus: vi
          .fn()
          .mockResolvedValue(workspaceStatus([renamed])),
        getWorkspaceFileDiff,
      }),
      "/repo",
      false,
      null,
      true,
    );

    expect(getWorkspaceFileDiff).toHaveBeenCalledWith(
      "/repo",
      "staged",
      "new.txt",
      true,
      "old.txt",
    );
  });

  it("describes workspace commit success effects", () => {
    expect(getWorkspaceCommitSuccessEffect()).toEqual({
      nextCommitMessage: "",
      refreshWorkspace: true,
      refreshRepository: true,
    });
  });

  it("keeps the commit message after workspace commit failure", () => {
    expect(getWorkspaceCommitFailureEffect("WIP")).toEqual({
      nextCommitMessage: "WIP",
      refreshWorkspace: false,
      refreshRepository: false,
    });
  });
});
