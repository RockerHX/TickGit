import { describe, expect, it } from "vitest";
import type { CommitListItem } from "$lib/types";
import { getErrorMessage, pickSelectedCommit } from "$lib/tickgit/page-helpers";

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

  it("picks selected commit with optional keep-selection behavior", () => {
    const commits = [commit("c3"), commit("c2"), commit("c1")];

    expect(pickSelectedCommit(commits, null, false)?.hash).toBe("c3");
    expect(pickSelectedCommit(commits, "c2", true)?.hash).toBe("c2");
    expect(pickSelectedCommit(commits, "missing", true)?.hash).toBe("c3");
  });
});
