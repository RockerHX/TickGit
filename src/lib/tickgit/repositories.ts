import type { RepositoryStatus, RepositorySummary } from "$lib/types";
import { isStepPushRunning } from "$lib/tickgit/page-state";
import type { StepPushUiState } from "$lib/types";

export type RepositoryManagementState = {
  loadingRepository: boolean;
  switchingBranch: boolean;
  isPushing: boolean;
  stepPushState: Pick<StepPushUiState, "status"> | null;
};

export function filterRepositories(
  repositories: RepositorySummary[],
  filterText: string,
) {
  const normalizedFilter = filterText.trim().toLowerCase();

  if (!normalizedFilter) {
    return repositories;
  }

  return repositories.filter((repository) => {
    return (
      repository.name.toLowerCase().includes(normalizedFilter) ||
      repository.path.toLowerCase().includes(normalizedFilter)
    );
  });
}

export function repositoryStatusLabel(status: RepositoryStatus) {
  switch (status) {
    case "available":
      return "Available";
    case "missing":
      return "Missing";
    case "invalid":
      return "Invalid";
  }
}

export function repositoryStatusTone(status: RepositoryStatus) {
  switch (status) {
    case "available":
      return "border-emerald-400/25 bg-emerald-400/10 text-emerald-200";
    case "missing":
      return "border-amber-300/25 bg-amber-300/10 text-amber-100";
    case "invalid":
      return "border-rose-400/25 bg-rose-400/10 text-rose-200";
  }
}

export function repositoryStatusMessage(repository: RepositorySummary) {
  if (repository.status === "available") {
    return null;
  }

  return (
    repository.disabledReason ??
    (repository.status === "missing"
      ? "仓库路径不存在"
      : "当前路径不是有效 Git 仓库")
  );
}

export function canManageRepositories(state: RepositoryManagementState) {
  return (
    !state.loadingRepository &&
    !state.switchingBranch &&
    !state.isPushing &&
    !isStepPushRunning(state.stepPushState)
  );
}
