import { describe, expect, it } from "vitest";
import type {
  BranchStatus,
  RepositorySummary,
  StepPushUiState,
} from "$lib/types";
import {
  canLoadCommitFiles,
  canLoadDiff,
  canLoadHistory,
  canPushCurrentBranch,
  canRefreshBlockedBranchStatus,
  canRefreshCurrentRepositoryOnFocus,
  canStartStepPush,
  canStartTargetCommitPush,
  canSwitchBranch,
  isBranchSwitcherDisabled,
  isContextMenuDisabled,
} from "$lib/tickgit/page-state";

function repository(): RepositorySummary {
  return {
    name: "repo",
    path: "/repo",
    lastOpenedAt: 1,
  };
}

function branchStatus(overrides: Partial<BranchStatus> = {}): BranchStatus {
  return {
    branch: "main",
    upstream: "origin/main",
    aheadCount: 1,
    safeAheadCount: 1,
    behindCount: 0,
    detached: false,
    pushAvailable: true,
    disabledReason: null,
    ...overrides,
  };
}

function stepPushState(status: StepPushUiState["status"]): StepPushUiState {
  return {
    jobId: 1,
    current: 1,
    total: 2,
    hash: "abc123",
    status,
  };
}

describe("page state", () => {
  it("allows branch switch only when repository is idle and target differs", () => {
    const base = {
      currentRepository: repository(),
      loadingRepository: false,
      switchingBranch: false,
      isPushing: false,
      stepPushState: null,
      branchStatus: branchStatus({ branch: "main" }),
    };

    expect(canSwitchBranch(base, "feature")).toBe(true);
    expect(canSwitchBranch(base, "main")).toBe(false);
    expect(
      canSwitchBranch({ ...base, currentRepository: null }, "feature"),
    ).toBe(false);
    expect(
      canSwitchBranch(
        { ...base, stepPushState: stepPushState("running") },
        "feature",
      ),
    ).toBe(false);
  });

  it("allows blocked branch refresh only when repository controls are idle", () => {
    const base = {
      currentRepository: repository(),
      loadingRepository: false,
      switchingBranch: false,
      isPushing: false,
      stepPushState: null,
    };

    expect(canRefreshBlockedBranchStatus(base)).toBe(true);
    expect(
      canRefreshBlockedBranchStatus({ ...base, loadingRepository: true }),
    ).toBe(false);
    expect(canRefreshBlockedBranchStatus({ ...base, isPushing: true })).toBe(
      false,
    );
    expect(
      canRefreshBlockedBranchStatus({
        ...base,
        stepPushState: stepPushState("running"),
      }),
    ).toBe(false);
  });

  it("allows current branch push only for available ahead branches", () => {
    const base = {
      branchStatus: branchStatus(),
      switchingBranch: false,
      isPushing: false,
      stepPushState: null,
    };

    expect(canPushCurrentBranch(base)).toBe(true);
    expect(
      canPushCurrentBranch({
        ...base,
        branchStatus: branchStatus({ aheadCount: 0 }),
      }),
    ).toBe(false);
    expect(
      canPushCurrentBranch({
        ...base,
        branchStatus: branchStatus({ pushAvailable: false }),
      }),
    ).toBe(false);
    expect(canPushCurrentBranch({ ...base, switchingBranch: true })).toBe(
      false,
    );
  });

  it("guards repository refresh and commit loading", () => {
    const repo = repository();

    expect(
      canRefreshCurrentRepositoryOnFocus({
        currentRepository: repo,
        loadingRepository: false,
        loadingHistory: false,
      }),
    ).toBe(true);
    expect(
      canRefreshCurrentRepositoryOnFocus({
        currentRepository: repo,
        loadingRepository: true,
        loadingHistory: false,
      }),
    ).toBe(false);
    expect(
      canLoadHistory({ currentRepository: repo, loadingHistory: false }),
    ).toBe(true);
    expect(
      canLoadHistory({ currentRepository: repo, loadingHistory: true }),
    ).toBe(false);
    expect(canLoadCommitFiles({ currentRepository: repo })).toBe(true);
    expect(canLoadCommitFiles({ currentRepository: null })).toBe(false);
    expect(canLoadDiff({ currentRepository: repo, selectedCommit: null })).toBe(
      false,
    );
  });

  it("guards commit push and step push actions", () => {
    const commit = {
      hash: "c1",
      shortHash: "c1",
      summary: "c1",
      authorName: "TickGit",
      authorEmail: "tickgit@example.com",
      committedAt: "2026-04-25T12:00:00Z",
      tags: [],
      parents: [],
      isPushed: false,
      isSafePushTarget: true,
      pushBlockedReason: null,
    };
    const base = {
      commit,
      currentRepository: repository(),
      branchStatus: branchStatus(),
      switchingBranch: false,
      isPushing: false,
      stepPushState: null,
    };

    expect(canStartTargetCommitPush(base)).toBe(true);
    expect(canStartTargetCommitPush({ ...base, isPushing: true })).toBe(false);
    expect(canStartStepPush(base)).toBe(true);
    expect(
      canStartStepPush({ ...base, stepPushState: stepPushState("running") }),
    ).toBe(false);
  });

  it("derives branch switcher and context menu disabled states", () => {
    const base = {
      currentRepository: repository(),
      loadingRepository: false,
      switchingBranch: false,
      isPushing: false,
      stepPushState: null,
    };

    expect(isBranchSwitcherDisabled(base)).toBe(false);
    expect(isBranchSwitcherDisabled({ ...base, currentRepository: null })).toBe(
      true,
    );
    expect(isContextMenuDisabled(base)).toBe(false);
    expect(isContextMenuDisabled({ ...base, isPushing: true })).toBe(true);
    expect(
      isContextMenuDisabled({
        ...base,
        stepPushState: stepPushState("running"),
      }),
    ).toBe(true);
  });
});
