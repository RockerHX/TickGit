import {
  FALLBACK_LOCALE,
  translate,
  translateErrorCode,
  type Locale,
} from "$lib/i18n";
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

export function formatRepositoryPath(path: string, homePath?: string | null) {
  const normalizedHome = homePath?.replace(/[\\/]+$/, "") ?? null;

  if (normalizedHome) {
    if (path === normalizedHome) {
      return "~";
    }

    if (path.startsWith(`${normalizedHome}/`)) {
      return `~/${path.slice(normalizedHome.length + 1)}`;
    }

    if (path.startsWith(`${normalizedHome}\\`)) {
      return `~\\${path.slice(normalizedHome.length + 1)}`;
    }
  }

  return path
    .replace(/^\/Users\/[^/]+(?=\/|$)/, "~")
    .replace(/^\/home\/[^/]+(?=\/|$)/, "~")
    .replace(/^[A-Za-z]:\\Users\\[^\\]+(?=\\|$)/, "~");
}

export function repositoryStatusBadgeLabel(status: RepositoryStatus) {
  switch (status) {
    case "available":
      return "ACTIVE";
    case "missing":
      return "MISSING";
    case "invalid":
      return "INVALID";
  }
}

export function repositoryStatusLabel(
  status: RepositoryStatus,
  locale: Locale = FALLBACK_LOCALE,
) {
  switch (status) {
    case "available":
      return translate(locale, "repository.status.available");
    case "missing":
      return translate(locale, "repository.status.missing");
    case "invalid":
      return translate(locale, "repository.status.invalid");
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

export function repositoryStatusMessage(
  repository: RepositorySummary,
  locale: Locale = FALLBACK_LOCALE,
) {
  if (repository.status === "available") {
    return null;
  }

  if (repository.disabledReasonCode) {
    return translateErrorCode(
      locale,
      repository.disabledReasonCode,
      repository.disabledReason,
    );
  }

  return (
    repository.disabledReason ??
    (repository.status === "missing"
      ? translate(locale, "repository.status.missingMessage")
      : translate(locale, "repository.status.invalidMessage"))
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
