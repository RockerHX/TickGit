import { describe, expect, it } from "vitest";
import type { CommitListItem } from "$lib/types";
import {
  buildStepPushHashes,
  getErrorMessage,
  pickSelectedCommit,
  toFailedStepPushState,
  toFinishedStepPushState,
  toRunningStepPushState,
} from "$lib/tickgit/page-helpers";

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

describe("page helpers", () => {
  it("extracts error messages from strings, app errors, and objects", () => {
    expect(getErrorMessage("plain error")).toBe("plain error");
    expect(getErrorMessage(new Error("native error"))).toBe("native error");
    expect(getErrorMessage({ message: "app error" })).toBe("app error");
    expect(getErrorMessage({ toString: () => "object error" })).toBe(
      "object error",
    );
    expect(getErrorMessage(null)).toBe("未知错误");
    expect(getErrorMessage(undefined)).toBe("未知错误");
  });

  it("builds step push hashes in old-to-new order", () => {
    const commits = [
      commit("pushed", true),
      commit("c3"),
      commit("c2"),
      commit("c1"),
    ];

    expect(buildStepPushHashes(commits, "c2")).toEqual(["c1", "c2"]);
    expect(buildStepPushHashes(commits, "missing")).toBeNull();
  });

  it("builds step push hashes for oldest and newest unpushed commits", () => {
    const commits = [
      commit("pushed", true),
      commit("c3"),
      commit("c2"),
      commit("c1"),
    ];

    expect(buildStepPushHashes(commits, "c1")).toEqual(["c1"]);
    expect(buildStepPushHashes(commits, "c3")).toEqual(["c1", "c2", "c3"]);
  });

  it("rejects pushed commits as step push targets", () => {
    const commits = [commit("pushed", true), commit("c2"), commit("c1")];

    expect(buildStepPushHashes(commits, "pushed")).toBeNull();
  });

  it("picks selected commit with optional keep-selection behavior", () => {
    const commits = [commit("c3"), commit("c2"), commit("c1")];

    expect(pickSelectedCommit(commits, null, false)?.hash).toBe("c3");
    expect(pickSelectedCommit(commits, "c2", true)?.hash).toBe("c2");
    expect(pickSelectedCommit(commits, "missing", true)?.hash).toBe("c3");
  });

  it("maps step push payloads into ui state", () => {
    expect(
      toRunningStepPushState({
        jobId: 3,
        current: 1,
        total: 2,
        hash: "abc",
      }),
    ).toEqual({
      jobId: 3,
      current: 1,
      total: 2,
      hash: "abc",
      status: "running",
    });

    expect(
      toFinishedStepPushState(
        {
          jobId: 3,
          total: 2,
        },
        {
          jobId: 3,
          current: 1,
          total: 2,
          hash: "abc",
          status: "running",
        },
      ),
    ).toEqual({
      jobId: 3,
      current: 2,
      total: 2,
      hash: "abc",
      status: "finished",
    });

    expect(
      toFailedStepPushState({
        jobId: 3,
        current: 2,
        total: 4,
        hash: "def",
        message: "push failed",
      }),
    ).toEqual({
      jobId: 3,
      current: 2,
      total: 4,
      hash: "def",
      status: "failed",
      message: "push failed",
    });
  });
});
