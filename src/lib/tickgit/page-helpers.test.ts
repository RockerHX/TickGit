import { describe, expect, it } from "vitest";
import type { CommitListItem } from "$lib/types";
import {
  getBranchSwitchErrorMessage,
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
    pushBlockedReasonCode: null,
  };
}

describe("page helpers", () => {
  it("extracts error messages from strings, app errors, and objects", () => {
    expect(getErrorMessage("plain error")).toBe("plain error");
    expect(getErrorMessage(new Error("native error"))).toBe("native error");
    expect(getErrorMessage({ message: "app error" })).toBe("app error");
    expect(
      getErrorMessage({ code: "repository_exists", message: "fallback" }),
    ).toBe("This repository is already in the list");
    expect(
      getErrorMessage(
        { code: "repository_exists", message: "fallback" },
        "zh-CN",
      ),
    ).toBe("该仓库已存在于列表中");
    expect(getErrorMessage({ code: "unknown_code", message: "fallback" })).toBe(
      "fallback",
    );
    expect(getErrorMessage({ toString: () => "object error" })).toBe(
      "object error",
    );
    expect(getErrorMessage(null)).toBe("Unknown error");
    expect(getErrorMessage(undefined, "zh-CN")).toBe("未知错误");
  });

  it("builds branch switch messages for structured checkout blockers", () => {
    expect(
      getBranchSwitchErrorMessage(
        { code: "checkout_blocked_by_local_changes", message: "fallback" },
        "feature/login",
      ),
    ).toBe(
      "Cannot switch to feature/login. Local uncommitted changes would be overwritten by the target branch. Git blocked the switch. Commit, stage, stash, or discard those changes and try again.",
    );
    expect(
      getBranchSwitchErrorMessage(
        { code: "checkout_blocked_by_untracked_files", message: "fallback" },
        "release",
        "zh-CN",
      ),
    ).toBe(
      "无法切换到 release。未跟踪文件会被目标分支覆盖，Git 已阻止切换。请先移动、删除、加入版本控制或 stash 这些文件后重试。",
    );
    expect(
      getBranchSwitchErrorMessage(
        { code: "unknown_code", message: "fallback" },
        "main",
        "zh-CN",
      ),
    ).toBe("fallback");
  });

  it("picks selected commit with optional keep-selection behavior", () => {
    const commits = [commit("c3"), commit("c2"), commit("c1")];

    expect(pickSelectedCommit(commits, null, false)?.hash).toBe("c3");
    expect(pickSelectedCommit(commits, "c2", true)?.hash).toBe("c2");
    expect(pickSelectedCommit(commits, "missing", true)?.hash).toBe("c3");
  });
});
