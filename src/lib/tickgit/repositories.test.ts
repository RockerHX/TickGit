import { describe, expect, it } from "vitest";
import type { RepositoryStatus, RepositorySummary } from "$lib/types";
import {
  canManageRepositories,
  filterRepositories,
  repositoryStatusLabel,
  repositoryStatusMessage,
  repositoryStatusTone,
} from "$lib/tickgit/repositories";

function repository(
  path: string,
  overrides: Partial<RepositorySummary> = {},
): RepositorySummary {
  return {
    name: path.split("/").at(-1) ?? path,
    path,
    lastOpenedAt: 1,
    status: "available",
    disabledReason: null,
    disabledReasonCode: null,
    ...overrides,
  };
}

describe("repository helpers", () => {
  it("filters repositories by name or path case-insensitively", () => {
    const repositories = [
      repository("/Users/tickgit/Alpha"),
      repository("/Users/tickgit/beta"),
      repository("/tmp/project-gamma"),
    ];

    expect(filterRepositories(repositories, "ALP")).toEqual([repositories[0]]);
    expect(filterRepositories(repositories, "PROJECT")).toEqual([
      repositories[2],
    ]);
    expect(filterRepositories(repositories, "  ")).toEqual(repositories);
  });

  it("maps repository status labels and tones", () => {
    const statuses: RepositoryStatus[] = ["available", "missing", "invalid"];

    expect(statuses.map((status) => repositoryStatusLabel(status))).toEqual([
      "Available",
      "Missing",
      "Invalid",
    ]);
    expect(repositoryStatusTone("available")).toContain("emerald");
    expect(repositoryStatusTone("missing")).toContain("amber");
    expect(repositoryStatusTone("invalid")).toContain("rose");
  });

  it("returns status messages for unavailable repositories", () => {
    expect(repositoryStatusMessage(repository("/repo"))).toBeNull();
    expect(
      repositoryStatusMessage(
        repository("/missing", {
          status: "missing",
          disabledReason: "仓库路径不存在",
          disabledReasonCode: "repository_missing",
        }),
      ),
    ).toBe("Repository path does not exist");
    expect(
      repositoryStatusMessage(
        repository(
          "/missing",
          {
            status: "missing",
            disabledReason: "仓库路径不存在",
            disabledReasonCode: "repository_missing",
          },
        ),
        "zh-CN",
      ),
    ).toBe("仓库路径不存在");
    expect(
      repositoryStatusMessage(
        repository("/invalid", {
          status: "invalid",
          disabledReason: null,
    disabledReasonCode: null,
        }),
      ),
    ).toBe("The current path is not a valid Git repository");
  });

  it("disables repository management while repository controls are busy", () => {
    const base = {
      loadingRepository: false,
      switchingBranch: false,
      isPushing: false,
      stepPushState: null,
    };

    expect(canManageRepositories(base)).toBe(true);
    expect(canManageRepositories({ ...base, loadingRepository: true })).toBe(
      false,
    );
    expect(canManageRepositories({ ...base, switchingBranch: true })).toBe(
      false,
    );
    expect(canManageRepositories({ ...base, isPushing: true })).toBe(false);
    expect(
      canManageRepositories({
        ...base,
        stepPushState: { status: "running" },
      }),
    ).toBe(false);
  });
});
