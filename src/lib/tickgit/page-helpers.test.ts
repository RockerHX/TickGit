import { describe, expect, it } from "vitest";
import type { CommitListItem } from "$lib/types";
import {
  buildStepPushHashes,
  getErrorMessage,
  pickSelectedCommit,
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
    isSafePushTarget: !isPushed,
    pushBlockedReason: null,
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

  it("rejects unsafe unpushed commits as step push targets", () => {
    const commits = [
      commit("unsafe-merge", false),
      commit("safe-2", false),
      commit("safe-1", false),
    ].map((item) =>
      item.hash === "unsafe-merge"
        ? {
            ...item,
            isSafePushTarget: false,
            pushBlockedReason:
              "该 Commit 未推送，但不在 first-parent 安全路径上，不能作为 step push / push to commit 目标",
          }
        : item,
    );

    expect(buildStepPushHashes(commits, "unsafe-merge")).toBeNull();
    expect(buildStepPushHashes(commits, "safe-2")).toEqual([
      "safe-1",
      "safe-2",
    ]);
  });

  it("picks selected commit with optional keep-selection behavior", () => {
    const commits = [commit("c3"), commit("c2"), commit("c1")];

    expect(pickSelectedCommit(commits, null, false)?.hash).toBe("c3");
    expect(pickSelectedCommit(commits, "c2", true)?.hash).toBe("c2");
    expect(pickSelectedCommit(commits, "missing", true)?.hash).toBe("c3");
  });


});
