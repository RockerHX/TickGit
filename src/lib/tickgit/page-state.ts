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

export type WorkspaceWriteState = RepositoryLoadState &
  PageBusyState & {
    loadingWorkspace: boolean;
    workspaceActionFileKey: string | null;
    committingWorkspace: boolean;
  };

export type WorkspaceCommitState = WorkspaceWriteState & {
  commitMessage: string;
  stagedCount: number;
};

export function isStepPushRunning(stepPushState: StepPushStatus) {
  return stepPushState?.status === "running";
}

export function isRepositoryAvailable(repository: RepositorySummary | null) {
  return repository?.status === "available";
}

export function shouldClearRepositoryData(
  repository: RepositorySummary | null,
) {
  return !isRepositoryAvailable(repository);
}

export function shouldShowRepositoryUnavailableState(
  repository: RepositorySummary | null,
) {
  return Boolean(repository && !isRepositoryAvailable(repository));
}

export function canSwitchBranch(
  state: BranchActionState,
  targetBranch: string,
) {
  return (
    isRepositoryAvailable(state.currentRepository) &&
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
    isRepositoryAvailable(state.currentRepository) &&
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
    isRepositoryAvailable(state.currentRepository) &&
    !state.loadingRepository &&
    !state.loadingHistory
  );
}

export function canLoadHistory(state: {
  currentRepository: RepositorySummary | null;
  loadingHistory: boolean;
}) {
  return (
    isRepositoryAvailable(state.currentRepository) && !state.loadingHistory
  );
}

export function canLoadCommitFiles(state: {
  currentRepository: RepositorySummary | null;
}) {
  return isRepositoryAvailable(state.currentRepository);
}

export function canLoadDiff(state: {
  currentRepository: RepositorySummary | null;
  selectedCommit: CommitListItem | null;
}) {
  return (
    isRepositoryAvailable(state.currentRepository) &&
    Boolean(state.selectedCommit)
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

export function canStartTargetCommitPush(state: CommitActionState) {
  return (
    Boolean(state.commit) &&
    isRepositoryAvailable(state.currentRepository) &&
    state.branchStatus?.pushAvailable === true &&
    !state.isPushing
  );
}

export function canStartStepPush(state: CommitActionState) {
  return (
    Boolean(state.commit) &&
    isRepositoryAvailable(state.currentRepository) &&
    state.branchStatus?.pushAvailable === true &&
    !isStepPushRunning(state.stepPushState)
  );
}

export function canWriteWorkspace(state: WorkspaceWriteState) {
  return (
    isRepositoryAvailable(state.currentRepository) &&
    !state.loadingRepository &&
    !state.loadingWorkspace &&
    !state.switchingBranch &&
    !state.isPushing &&
    !state.workspaceActionFileKey &&
    !state.committingWorkspace &&
    !isStepPushRunning(state.stepPushState)
  );
}

export function canCreateWorkspaceCommit(state: WorkspaceCommitState) {
  return (
    canWriteWorkspace(state) &&
    state.stagedCount > 0 &&
    state.commitMessage.trim().length > 0
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
