import type {
  BranchStatus,
  CommitListItem,
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

export type BranchActionState = RepositoryLoadState &
  PageBusyState & { branchStatus: BranchStatus | null };

export type CommitActionState = PageBusyState & {
  currentRepository: RepositorySummary | null;
  branchStatus: BranchStatus | null;
  commit: CommitListItem | null;
};

export function isStepPushRunning(stepPushState: StepPushStatus) {
  return stepPushState?.status === "running";
}

export function canSwitchBranch(
  state: BranchActionState,
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

export function canRefreshCurrentRepositoryOnFocus(
  state: RepositoryLoadState & { loadingHistory: boolean },
) {
  return (
    Boolean(state.currentRepository) &&
    !state.loadingRepository &&
    !state.loadingHistory
  );
}

export function canLoadHistory(state: {
  currentRepository: RepositorySummary | null;
  loadingHistory: boolean;
}) {
  return Boolean(state.currentRepository) && !state.loadingHistory;
}

export function canLoadCommitFiles(state: {
  currentRepository: RepositorySummary | null;
}) {
  return Boolean(state.currentRepository);
}

export function canLoadDiff(state: {
  currentRepository: RepositorySummary | null;
  selectedCommit: CommitListItem | null;
}) {
  return Boolean(state.currentRepository) && Boolean(state.selectedCommit);
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

export function canStartTargetCommitPush(state: CommitActionState) {
  return (
    Boolean(state.commit) &&
    Boolean(state.currentRepository) &&
    state.branchStatus?.pushAvailable === true &&
    !state.isPushing
  );
}

export function canStartStepPush(state: CommitActionState) {
  return (
    Boolean(state.commit) &&
    Boolean(state.currentRepository) &&
    state.branchStatus?.pushAvailable === true &&
    !isStepPushRunning(state.stepPushState)
  );
}

export function isBranchSwitcherDisabled(
  state: RepositoryLoadState & PageBusyState,
) {
  return !canRefreshBlockedBranchStatus(state);
}

export function isContextMenuDisabled(state: PageBusyState) {
  return (
    state.switchingBranch ||
    state.isPushing ||
    isStepPushRunning(state.stepPushState)
  );
}
