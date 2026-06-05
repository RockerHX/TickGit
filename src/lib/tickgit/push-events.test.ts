import { describe, expect, it } from "vitest";
import type { PushToCommitUiState, StepPushUiState } from "$lib/types";
import {
  canManuallyDismissOverlay,
  dismissFailedOverlay,
  dismissOverlayIfJobMatches,
  formatPushTargetLabel,
  toFailedPushToCommitState,
  toFailedStepPushState,
  toFinishedPushToCommitState,
  toFinishedStepPushState,
  toRunningPushToCommitState,
  toRunningStepPushState,
} from "$lib/tickgit/push-events";

function pushState(
  overrides: Partial<PushToCommitUiState> = {},
): PushToCommitUiState {
  return {
    jobId: 7,
    target: "abc1234",
    targetKind: "commit",
    status: "running",
    ...overrides,
  };
}

function stepState(overrides: Partial<StepPushUiState> = {}): StepPushUiState {
  return {
    jobId: 9,
    current: 1,
    total: 3,
    hash: "def5678",
    status: "running",
    ...overrides,
  };
}

describe("push events", () => {
  it("formats commit and branch push targets", () => {
    expect(formatPushTargetLabel("1234567890", "commit")).toEqual({
      inline: "1234567",
      message: "Commit 1234567",
    });
    expect(formatPushTargetLabel("1234567890", "commit", "zh-CN")).toEqual({
      inline: "1234567",
      message: "Commit 1234567",
    });
    expect(formatPushTargetLabel("origin/main", "branch")).toEqual({
      inline: "origin/main",
      message: "origin/main",
    });
  });

  it("maps push to commit payloads into ui state", () => {
    expect(
      toRunningPushToCommitState({
        jobId: 1,
        target: "1234567890",
        targetKind: "commit",
      }),
    ).toEqual({
      jobId: 1,
      target: "1234567",
      targetKind: "commit",
      status: "running",
    });

    expect(
      toFinishedPushToCommitState({
        jobId: 1,
        target: "origin/main",
        targetKind: "branch",
      }),
    ).toEqual({
      jobId: 1,
      target: "origin/main",
      targetKind: "branch",
      status: "finished",
    });

    expect(
      toFailedPushToCommitState({
        jobId: 1,
        target: "1234567890",
        targetKind: "commit",
        message: "push failed",
        code: "behind_remote",
      }),
    ).toEqual({
      jobId: 1,
      target: "1234567",
      targetKind: "commit",
      status: "failed",
      message:
        "Remote has updates. TickGit cannot push safely yet. Sync the remote with GitHub Desktop or SourceTree, then return to TickGit and refresh.",
      code: "behind_remote",
    });
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
      toRunningStepPushState({
        jobId: 3,
        total: 2,
        hash: "abc",
      }),
    ).toEqual({
      jobId: 3,
      current: 0,
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
      toFailedStepPushState(
        {
          jobId: 3,
          current: 2,
          total: 4,
          hash: "def",
          message: "push failed",
          code: "unsafe_push_target",
        },
        "zh-CN",
      ),
    ).toEqual({
      jobId: 3,
      current: 2,
      total: 4,
      hash: "def",
      status: "failed",
      message:
        "该 Commit 未推送，但不在 first-parent 安全路径上，不能作为 step push / push to commit 目标。",
      code: "unsafe_push_target",
    });
  });

  it("shows manual close only for failed overlays", () => {
    expect(canManuallyDismissOverlay(pushState({ status: "running" }))).toBe(
      false,
    );
    expect(canManuallyDismissOverlay(pushState({ status: "finished" }))).toBe(
      false,
    );
    expect(canManuallyDismissOverlay(stepState({ status: "failed" }))).toBe(
      true,
    );
    expect(canManuallyDismissOverlay(null)).toBe(false);
  });

  it("dismisses only failed overlays when user closes them", () => {
    expect(dismissFailedOverlay(pushState({ status: "failed" }))).toBeNull();
    expect(dismissFailedOverlay(stepState({ status: "failed" }))).toBeNull();

    const runningPush = pushState({ status: "running" });
    const finishedStep = stepState({ status: "finished" });

    expect(dismissFailedOverlay(runningPush)).toBe(runningPush);
    expect(dismissFailedOverlay(finishedStep)).toBe(finishedStep);
    expect(dismissFailedOverlay(null)).toBeNull();
  });

  it("auto-dismisses only the matching overlay job", () => {
    const failedPush = pushState({ jobId: 11, status: "failed" });
    const runningStep = stepState({ jobId: 12, status: "running" });

    expect(dismissOverlayIfJobMatches(failedPush, 11)).toBeNull();
    expect(dismissOverlayIfJobMatches(runningStep, 12)).toBeNull();
    expect(dismissOverlayIfJobMatches(failedPush, 99)).toBe(failedPush);
    expect(dismissOverlayIfJobMatches(null, 11)).toBeNull();
  });
});
