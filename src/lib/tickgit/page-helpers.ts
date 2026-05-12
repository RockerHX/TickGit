import type {
  AppError,
  CommitListItem,
  PushToCommitUiState,
  StepPushFailed,
  StepPushFinished,
  StepPushProgress,
  StepPushUiState,
  ToastItem,
} from "$lib/types";

export function getErrorMessage(error: unknown) {
  if (typeof error === "string") {
    return error;
  }

  if (error && typeof error === "object") {
    const appError = error as Partial<AppError>;
    if (typeof appError.message === "string") {
      return appError.message;
    }

    if ("toString" in error && typeof error.toString === "function") {
      return error.toString();
    }
  }

  return "未知错误";
}

export function buildStepPushHashes(
  commits: CommitListItem[],
  targetHash: string,
) {
  // 历史列表现在是全量口径；分步推送只能从后端标记过的 first-parent 安全路径里取 hash。
  const safeCommits = commits.filter((item) => item.isSafePushTarget);
  const targetIndex = safeCommits.findIndex((item) => item.hash === targetHash);

  if (targetIndex === -1) {
    return null;
  }

  return safeCommits
    .slice(targetIndex)
    .reverse()
    .map((item) => item.hash);
}

export function pickSelectedCommit(
  commits: CommitListItem[],
  previousSelectedHash: string | null,
  keepSelection: boolean,
) {
  if (!keepSelection || !previousSelectedHash) {
    return commits[0] ?? null;
  }

  return (
    commits.find((item) => item.hash === previousSelectedHash) ??
    commits[0] ??
    null
  );
}

export function createToastItem(
  id: number,
  title: string,
  message: string,
  tone: ToastItem["tone"] = "info",
): ToastItem {
  return { id, title, message, tone };
}

type PushOverlayUiState = PushToCommitUiState | StepPushUiState;

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

export function toRunningStepPushState(
  payload: Pick<StepPushProgress, "jobId" | "current" | "total" | "hash">,
): StepPushUiState {
  return {
    jobId: payload.jobId,
    current: payload.current,
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
