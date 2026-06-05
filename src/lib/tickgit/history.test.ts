import { describe, expect, it } from "vitest";
import type { CommitFileChange } from "$lib/types";
import {
  commitFileMatchesPathFilter,
  getActiveHistoryFilterCount,
  hasActiveHistoryFilters,
  historyFiltersEqual,
  normalizeHistoryFilters,
  pickCommitFileForPathFilter,
} from "$lib/tickgit/history";

function fileChange(
  path: string,
  overrides: Partial<CommitFileChange> = {},
): CommitFileChange {
  return {
    status: "M",
    path,
    previousPath: null,
    displayPath: path,
    ...overrides,
  };
}

describe("history filters", () => {
  it("normalizes history filters and counts active fields", () => {
    const filters = normalizeHistoryFilters({
      query: "  fix ",
      author: " ",
      filePath: " src/app ",
    });

    expect(filters).toEqual({
      query: "fix",
      author: "",
      filePath: "src/app",
    });
    expect(getActiveHistoryFilterCount(filters)).toBe(2);
    expect(hasActiveHistoryFilters(filters)).toBe(true);
    expect(getActiveHistoryFilterCount(null)).toBe(0);
  });

  it("compares normalized history filters", () => {
    expect(
      historyFiltersEqual(
        { query: " fix ", author: "Ada", filePath: "" },
        { query: "fix", author: "Ada", filePath: null },
      ),
    ).toBe(true);
    expect(
      historyFiltersEqual(
        { query: "fix", author: "Ada" },
        { query: "fix", author: "Bob" },
      ),
    ).toBe(false);
  });

  it("matches file paths by current, previous, and display paths", () => {
    const renamed = fileChange("src/new-name.ts", {
      previousPath: "src/old-name.ts",
      displayPath: "src/old-name.ts -> src/new-name.ts",
    });

    expect(commitFileMatchesPathFilter(renamed, "NEW-NAME")).toBe(true);
    expect(commitFileMatchesPathFilter(renamed, "old-name")).toBe(true);
    expect(commitFileMatchesPathFilter(renamed, "old-name.ts -> src/new")).toBe(
      true,
    );
    expect(commitFileMatchesPathFilter(renamed, "missing")).toBe(false);
  });

  it("prefers the first file matching the path filter", () => {
    const files = [
      fileChange("README.md"),
      fileChange("src/main.ts"),
      fileChange("src/app.ts"),
    ];

    expect(pickCommitFileForPathFilter(files, "src/app")?.path).toBe(
      "src/app.ts",
    );
    expect(pickCommitFileForPathFilter(files, "missing")?.path).toBe(
      "README.md",
    );
    expect(pickCommitFileForPathFilter([], "src")).toBe(null);
  });
});
