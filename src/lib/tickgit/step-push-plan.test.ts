import { describe, expect, it, vi } from "vitest";
import type { StepPushPlan } from "$lib/types";
import {
  getStepPushPlanBlockedMessage,
  getStepPushPlanHashes,
  startStepPushFromPlan,
} from "$lib/tickgit/step-push-plan";

function availablePlan(): StepPushPlan {
  return {
    branch: "main",
    targetHash: "c2",
    available: true,
    items: [
      { hash: "c1", shortHash: "c1", summary: "first" },
      { hash: "c2", shortHash: "c2", summary: "second" },
    ],
    blockedReason: null,
  };
}

describe("step push plan", () => {
  it("keeps backend plan order when extracting hashes", () => {
    expect(getStepPushPlanHashes(availablePlan())).toEqual(["c1", "c2"]);
  });

  it("starts step push from an available backend plan", async () => {
    const api = {
      startStepPush: vi.fn().mockResolvedValue({ jobId: 7, total: 2 }),
    };

    const state = await startStepPushFromPlan(api, availablePlan(), "/repo");

    expect(api.startStepPush).toHaveBeenCalledWith({
      repoPath: "/repo",
      branch: "main",
      hashes: ["c1", "c2"],
      delayMs: 1500,
    });
    expect(state).toEqual({
      jobId: 7,
      current: 0,
      total: 2,
      hash: "c1",
      status: "running",
    });
  });

  it("does not start step push when plan is blocked", async () => {
    const api = {
      startStepPush: vi.fn().mockResolvedValue({ jobId: 7, total: 2 }),
    };
    const plan: StepPushPlan = {
      ...availablePlan(),
      available: false,
      items: [],
      blockedReason: {
        code: "behind_remote",
        message: "远端已有更新",
      },
    };

    await expect(startStepPushFromPlan(api, plan, "/repo")).rejects.toThrow(
      "远端已有更新",
    );
    expect(api.startStepPush).not.toHaveBeenCalled();
    expect(getStepPushPlanBlockedMessage(plan)).toBe("远端已有更新");
  });
});
