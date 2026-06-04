import type {
  StepPushJobStarted,
  StepPushPlan,
  StepPushRequest,
  StepPushUiState,
} from "$lib/types";
import { toRunningStepPushState } from "$lib/tickgit/push-events";

export type StepPushPlanApi = {
  startStepPush(request: StepPushRequest): Promise<StepPushJobStarted>;
};

export function getStepPushPlanHashes(plan: StepPushPlan) {
  return plan.items.map((item) => item.hash);
}

export function getStepPushPlanBlockedMessage(plan: StepPushPlan) {
  return plan.blockedReason?.message ?? "目标 Commit 当前不可分步推送";
}

export async function startStepPushFromPlan(
  api: StepPushPlanApi,
  plan: StepPushPlan,
  repoPath: string,
  delayMs = 1500,
): Promise<StepPushUiState> {
  const hashes = getStepPushPlanHashes(plan);

  if (!plan.available || hashes.length === 0) {
    throw new Error(getStepPushPlanBlockedMessage(plan));
  }

  const started = await api.startStepPush({
    repoPath,
    branch: plan.branch,
    hashes,
    delayMs,
  });

  return toRunningStepPushState({
    ...started,
    hash: hashes[0],
  });
}
