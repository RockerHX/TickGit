import type {
  BranchStatus,
  RepositorySummary,
  StepPushUiState,
} from "$lib/types";

type StepPushStatus = Pick<StepPushUiState, "status"> | null;

export type PageBusyState = {
  switchingBranch: boolean;
  isPushing: boolean;
  stepPushState: StepPushStatus;
};

export type RepositoryLoadState = {
  currentRepository: RepositorySummary | null;
  loadingRepository: boolean;
};

export function isStepPushRunning(stepPushState: StepPushStatus) {
  return stepPushState?.status === "running";
}

export function canSwitchBranch(
  state: RepositoryLoadState & PageBusyState & { branchStatus: BranchStatus | null },
  targetBranch: string,
) {
  return (
    Boolean(state.currentRepository) &&
    !state.loadingRepository &&
    !state.switchingBranch &&
    !state.isPushing &&
    !isStepPushRunning(state.stepPushState) &&
    targetBranch !== state.branchStatus?.branch
  );
}

export function canRefreshBlockedBranchStatus(
  state: RepositoryLoadState & PageBusyState,
) {
  return (
    Boolean(state.currentRepository) &&
    !state.loadingRepository &&
    !state.switchingBranch &&
    !state.isPushing &&
    !isStepPushRunning(state.stepPushState)
  );
}

export function canPushCurrentBranch(
  state: PageBusyState & { branchStatus: BranchStatus | null },
) {
  return (
    state.branchStatus?.pushAvailable === true &&
    state.branchStatus.aheadCount > 0 &&
    !state.switchingBranch &&
    !state.isPushing &&
    !isStepPushRunning(state.stepPushState)
  );
}

export function isBranchSwitcherDisabled(state: RepositoryLoadState & PageBusyState) {
  return !canRefreshBlockedBranchStatus(state);
}

export function isContextMenuDisabled(state: PageBusyState) {
  return (
    state.switchingBranch || state.isPushing || isStepPushRunning(state.stepPushState)
  );
}
