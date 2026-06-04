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
): PushTargetLabel {
  if (targetKind === "commit") {
    const shortHash = target.length > 7 ? target.slice(0, 7) : target;
    return {
      inline: shortHash,
      message: `Commit ${shortHash}`,
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
): PushToCommitUiState {
  const target = formatPushTargetLabel(payload.target, payload.targetKind);

  return {
    jobId: payload.jobId,
    target: target.inline,
    targetKind: payload.targetKind,
    status: "failed",
    message: payload.message,
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
): StepPushUiState {
  return {
    jobId: payload.jobId,
    current: payload.current,
    total: payload.total,
    hash: payload.hash,
    status: "failed",
    message: payload.message,
  };
}
