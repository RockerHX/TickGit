import { describe, expect, it } from "vitest";
import { repositoryContextMenuActionMap } from "$lib/tickgit/repository-context-menu";
import type { RepositorySummary } from "$lib/types";

function repository(
  status: RepositorySummary["status"] = "available",
): RepositorySummary {
  return {
    name: "TickGit",
    path: "/tmp/TickGit",
    lastOpenedAt: 1,
    status,
    disabledReason: status === "available" ? null : "Unavailable",
  };
}

describe("repository context menu", () => {
  it("enables all base actions for an available repository with GitHub URL", () => {
    const actions = repositoryContextMenuActionMap({
      repository: repository(),
      githubUrl: "https://github.com/example/TickGit",
      githubLoading: false,
      managementDisabled: false,
    });

    expect(actions.copyName.disabled).toBe(false);
    expect(actions.copyPath.disabled).toBe(false);
    expect(actions.viewGithub.disabled).toBe(false);
    expect(actions.openTerminal.disabled).toBe(false);
    expect(actions.revealInFinder.disabled).toBe(false);
    expect(actions.openInVSCode.disabled).toBe(false);
    expect(actions.remove.disabled).toBe(false);
  });

  it.each(["missing", "invalid"] as const)(
    "only allows copy and remove actions for a %s repository",
    (status) => {
      const actions = repositoryContextMenuActionMap({
        repository: repository(status),
        githubUrl: "https://github.com/example/TickGit",
        githubLoading: false,
        managementDisabled: false,
      });

      expect(actions.copyName.disabled).toBe(false);
      expect(actions.copyPath.disabled).toBe(false);
      expect(actions.viewGithub.disabled).toBe(true);
      expect(actions.openTerminal.disabled).toBe(true);
      expect(actions.revealInFinder.disabled).toBe(true);
      expect(actions.openInVSCode.disabled).toBe(true);
      expect(actions.remove.disabled).toBe(false);
    },
  );

  it("disables remove while repository management is disabled", () => {
    const actions = repositoryContextMenuActionMap({
      repository: repository(),
      githubUrl: "https://github.com/example/TickGit",
      githubLoading: false,
      managementDisabled: true,
    });

    expect(actions.remove.disabled).toBe(true);
    expect(actions.remove.disabledReasonKey).toBe(
      "repository.contextManagementDisabled",
    );
  });

  it("disables GitHub action without a URL or while loading", () => {
    const withoutUrl = repositoryContextMenuActionMap({
      repository: repository(),
      githubUrl: null,
      githubLoading: false,
      managementDisabled: false,
    });
    const loading = repositoryContextMenuActionMap({
      repository: repository(),
      githubUrl: "https://github.com/example/TickGit",
      githubLoading: true,
      managementDisabled: false,
    });

    expect(withoutUrl.viewGithub.disabled).toBe(true);
    expect(withoutUrl.viewGithub.disabledReasonKey).toBe(
      "repository.contextGithubUnavailable",
    );
    expect(loading.viewGithub.disabled).toBe(true);
    expect(loading.viewGithub.disabledReasonKey).toBe(
      "repository.contextGithubLoading",
    );
  });

  it("disables every action without a repository", () => {
    const actions = repositoryContextMenuActionMap({
      repository: null,
      githubUrl: null,
      githubLoading: false,
      managementDisabled: false,
    });

    expect(Object.values(actions).every((action) => action.disabled)).toBe(
      true,
    );
  });
});
