import type { RepositorySummary } from "$lib/types";

export type RepositoryContextMenuActionId =
  | "copyName"
  | "copyPath"
  | "viewGithub"
  | "openTerminal"
  | "revealInFinder"
  | "openInVSCode"
  | "remove";

export type RepositoryContextMenuActionState = {
  id: RepositoryContextMenuActionId;
  disabled: boolean;
  disabledReasonKey: string | null;
};

export type RepositoryContextMenuStateInput = {
  repository: RepositorySummary | null;
  githubUrl: string | null;
  githubLoading: boolean;
  managementDisabled: boolean;
};

const ACTION_IDS: RepositoryContextMenuActionId[] = [
  "copyName",
  "copyPath",
  "viewGithub",
  "openTerminal",
  "revealInFinder",
  "openInVSCode",
  "remove",
];

function actionState(
  id: RepositoryContextMenuActionId,
  disabled: boolean,
  disabledReasonKey: string | null = null,
): RepositoryContextMenuActionState {
  return {
    id,
    disabled,
    disabledReasonKey: disabled ? disabledReasonKey : null,
  };
}

export function getRepositoryContextMenuActionStates({
  repository,
  githubUrl,
  githubLoading,
  managementDisabled,
}: RepositoryContextMenuStateInput): RepositoryContextMenuActionState[] {
  if (!repository) {
    return ACTION_IDS.map((id) =>
      actionState(id, true, "repository.contextUnavailable"),
    );
  }

  const repositoryAvailable = repository.status === "available";
  const unavailableReason = repositoryAvailable
    ? null
    : "repository.contextPathUnavailable";

  return [
    actionState("copyName", false),
    actionState("copyPath", false),
    actionState(
      "viewGithub",
      githubLoading || !repositoryAvailable || !githubUrl,
      githubLoading
        ? "repository.contextGithubLoading"
        : (unavailableReason ?? "repository.contextGithubUnavailable"),
    ),
    actionState("openTerminal", !repositoryAvailable, unavailableReason),
    actionState("revealInFinder", !repositoryAvailable, unavailableReason),
    actionState("openInVSCode", !repositoryAvailable, unavailableReason),
    actionState(
      "remove",
      managementDisabled,
      "repository.contextManagementDisabled",
    ),
  ];
}

export function repositoryContextMenuActionMap(
  input: RepositoryContextMenuStateInput,
) {
  return Object.fromEntries(
    getRepositoryContextMenuActionStates(input).map((state) => [
      state.id,
      state,
    ]),
  ) as Record<RepositoryContextMenuActionId, RepositoryContextMenuActionState>;
}
