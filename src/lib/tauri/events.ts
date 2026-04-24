import { getCurrentWindow } from "@tauri-apps/api/window";
import type {
  StepPushFailed,
  StepPushFinished,
  StepPushProgress,
} from "$lib/types";

export const STEP_PUSH_PROGRESS_EVENT = "step-push-progress";
export const STEP_PUSH_FINISHED_EVENT = "step-push-finished";
export const STEP_PUSH_FAILED_EVENT = "step-push-failed";

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
