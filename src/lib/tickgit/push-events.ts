import {
  FALLBACK_LOCALE,
  translate,
  translateErrorCode,
  type Locale,
} from "$lib/i18n";
import type {
  PushTargetKind,
  PushToCommitFailed,
  PushToCommitFinished,
  PushToCommitJobStarted,
  PushToCommitUiState,
  StepPushFailed,
  StepPushFinished,
  StepPushJobStarted,
  StepPushProgress,
  StepPushUiState,
} from "$lib/types";

export type PushTargetLabel = {
  inline: string;
  message: string;
};

type PushOverlayUiState = PushToCommitUiState | StepPushUiState;

export function formatPushTargetLabel(
  target: string,
  targetKind: PushTargetKind,
  locale: Locale = FALLBACK_LOCALE,
): PushTargetLabel {
  if (targetKind === "commit") {
    const shortHash = target.length > 7 ? target.slice(0, 7) : target;
    return {
      inline: shortHash,
      message: translate(locale, "push.targetCommit", { hash: shortHash }),
    };
  }

  return {
    inline: target,
    message: target,
  };
}

export function canManuallyDismissOverlay(
  state: Pick<PushOverlayUiState, "status"> | null,
) {
  return state?.status === "failed";
}

export function dismissFailedOverlay<
  T extends Pick<PushOverlayUiState, "status">,
>(state: T | null) {
  return canManuallyDismissOverlay(state) ? null : state;
}

export function dismissOverlayIfJobMatches<
  T extends Pick<PushOverlayUiState, "jobId">,
>(state: T | null, jobId: number) {
  return state?.jobId === jobId ? null : state;
}

export function toRunningPushToCommitState(
  payload: Pick<PushToCommitJobStarted, "jobId" | "target" | "targetKind">,
): PushToCommitUiState {
  const target = formatPushTargetLabel(payload.target, payload.targetKind);

  return {
    jobId: payload.jobId,
    target: target.inline,
    targetKind: payload.targetKind,
    status: "running",
  };
}

export function toFinishedPushToCommitState(
  payload: PushToCommitFinished,
): PushToCommitUiState {
  const target = formatPushTargetLabel(payload.target, payload.targetKind);

  return {
    jobId: payload.jobId,
    target: target.inline,
    targetKind: payload.targetKind,
    status: "finished",
  };
}

export function toFailedPushToCommitState(
  payload: PushToCommitFailed,
  locale: Locale = FALLBACK_LOCALE,
): PushToCommitUiState {
  const target = formatPushTargetLabel(
    payload.target,
    payload.targetKind,
    locale,
  );

  return {
    jobId: payload.jobId,
    target: target.inline,
    targetKind: payload.targetKind,
    status: "failed",
    message: translateErrorCode(locale, payload.code, payload.message),
    ...(payload.code ? { code: payload.code } : {}),
  };
}

export function toRunningStepPushState(
  payload:
    | (Pick<StepPushJobStarted, "jobId" | "total"> & { hash: string })
    | Pick<StepPushProgress, "jobId" | "current" | "total" | "hash">,
): StepPushUiState {
  return {
    jobId: payload.jobId,
    current: "current" in payload ? payload.current : 0,
    total: payload.total,
    hash: payload.hash,
    status: "running",
  };
}

export function toFinishedStepPushState(
  payload: StepPushFinished,
  currentState: StepPushUiState | null,
): StepPushUiState {
  return {
    jobId: payload.jobId,
    current: payload.total,
    total: payload.total,
    hash: currentState?.hash ?? "",
    status: "finished",
  };
}

export function toFailedStepPushState(
  payload: StepPushFailed,
  locale: Locale = FALLBACK_LOCALE,
): StepPushUiState {
  return {
    jobId: payload.jobId,
    current: payload.current,
    total: payload.total,
    hash: payload.hash,
    status: "failed",
    message: translateErrorCode(locale, payload.code, payload.message),
    ...(payload.code ? { code: payload.code } : {}),
  };
}
