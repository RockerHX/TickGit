import { getCurrentWindow } from "@tauri-apps/api/window";
import type {
  PushToCommitFailed,
  PushToCommitFinished,
  StepPushFailed,
  StepPushFinished,
  StepPushProgress,
} from "$lib/types";

export const STEP_PUSH_PROGRESS_EVENT = "step-push-progress";
export const STEP_PUSH_FINISHED_EVENT = "step-push-finished";
export const STEP_PUSH_FAILED_EVENT = "step-push-failed";
export const PUSH_TO_COMMIT_FINISHED_EVENT = "push-to-commit-finished";
export const PUSH_TO_COMMIT_FAILED_EVENT = "push-to-commit-failed";

export async function listenStepPushProgress(
  handler: (payload: StepPushProgress) => void,
) {
  return getCurrentWindow().listen<StepPushProgress>(
    STEP_PUSH_PROGRESS_EVENT,
    (event) => {
      handler(event.payload);
    },
  );
}

export async function listenStepPushFinished(
  handler: (payload: StepPushFinished) => void,
) {
  return getCurrentWindow().listen<StepPushFinished>(
    STEP_PUSH_FINISHED_EVENT,
    (event) => {
      handler(event.payload);
    },
  );
}

export async function listenStepPushFailed(
  handler: (payload: StepPushFailed) => void,
) {
  return getCurrentWindow().listen<StepPushFailed>(
    STEP_PUSH_FAILED_EVENT,
    (event) => {
      handler(event.payload);
    },
  );
}

export async function listenPushToCommitFinished(
  handler: (payload: PushToCommitFinished) => void,
) {
  return getCurrentWindow().listen<PushToCommitFinished>(
    PUSH_TO_COMMIT_FINISHED_EVENT,
    (event) => {
      handler(event.payload);
    },
  );
}

export async function listenPushToCommitFailed(
  handler: (payload: PushToCommitFailed) => void,
) {
  return getCurrentWindow().listen<PushToCommitFailed>(
    PUSH_TO_COMMIT_FAILED_EVENT,
    (event) => {
      handler(event.payload);
    },
  );
}
