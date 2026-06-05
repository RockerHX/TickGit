import { FALLBACK_LOCALE, translate, translateErrorCode, type Locale } from "$lib/i18n";
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

export function getStepPushPlanBlockedMessage(
  plan: StepPushPlan,
  locale: Locale = FALLBACK_LOCALE,
) {
  if (plan.blockedReason) {
    return translateErrorCode(
      locale,
      plan.blockedReason.code,
      plan.blockedReason.message,
    );
  }

  return translate(locale, "history.unsafeStepPushFallback");
}

export async function startStepPushFromPlan(
  api: StepPushPlanApi,
  plan: StepPushPlan,
  repoPath: string,
  delayMs = 1500,
  locale: Locale = FALLBACK_LOCALE,
): Promise<StepPushUiState> {
  const hashes = getStepPushPlanHashes(plan);

  if (!plan.available || hashes.length === 0) {
    throw new Error(getStepPushPlanBlockedMessage(plan, locale));
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
