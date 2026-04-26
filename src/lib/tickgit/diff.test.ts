import { describe, expect, it } from "vitest";
import {
  buildSplitDiffRows,
  getDiffViewerState,
  parseUnifiedDiff,
} from "$lib/tickgit/diff";

describe("diff parser", () => {
  it("returns an empty diff for empty input", () => {
    expect(parseUnifiedDiff("")).toEqual({
      hunks: [],
      isEmpty: true,
      maxLineNumberWidth: 1,
      parseError: false,
    });
  });

  it("parses a single hunk with context, additions, and deletions", () => {
    const diff = parseUnifiedDiff(
      [
        "diff --git a/file.txt b/file.txt",
        "--- a/file.txt",
        "+++ b/file.txt",
        "@@ -1,2 +1,2 @@",
        " hello",
        "-before",
        "+after",
      ].join("\n"),
    );

    expect(diff.parseError).toBe(false);
    expect(diff.hunks).toHaveLength(1);
    expect(diff.hunks[0].lines).toEqual([
      expect.objectContaining({
        type: "context",
        content: "hello",
        oldLineNumber: 1,
        newLineNumber: 1,
      }),
      expect.objectContaining({
        type: "delete",
        content: "before",
        oldLineNumber: 2,
        newLineNumber: null,
      }),
      expect.objectContaining({
        type: "add",
        content: "after",
        oldLineNumber: null,
        newLineNumber: 2,
      }),
    ]);
  });

  it("parses multiple hunks and keeps line numbering per hunk", () => {
    const diff = parseUnifiedDiff(
      [
        "@@ -1,1 +1,1 @@",
        "-before",
        "+after",
        "@@ -10,2 +10,3 @@",
        " keep",
        "-old",
        "+new",
        "+plus",
      ].join("\n"),
    );

    expect(diff.hunks).toHaveLength(2);
    expect(diff.hunks[1].lines[0]).toEqual(
      expect.objectContaining({
        type: "context",
        oldLineNumber: 10,
        newLineNumber: 10,
      }),
    );
    expect(diff.hunks[1].lines[2]).toEqual(
      expect.objectContaining({
        type: "add",
        oldLineNumber: null,
        newLineNumber: 11,
      }),
    );
    expect(diff.hunks[1].lines[3]).toEqual(
      expect.objectContaining({
        type: "add",
        oldLineNumber: null,
        newLineNumber: 12,
      }),
    );
  });

  it("marks no trailing newline on the preceding line", () => {
    const diff = parseUnifiedDiff(
      [
        "@@ -1,1 +1,1 @@",
        "-before",
        "+after",
        "\\ No newline at end of file",
      ].join("\n"),
    );

    expect(diff.hunks[0].lines[1].noTrailingNewLine).toBe(true);
  });

  it("parses initial commit style patches", () => {
    const diff = parseUnifiedDiff(
      [
        "diff --git a/file.txt b/file.txt",
        "new file mode 100644",
        "--- /dev/null",
        "+++ b/file.txt",
        "@@ -0,0 +1,2 @@",
        "+hello",
        "+world",
      ].join("\n"),
    );

    expect(diff.hunks[0].lines[0]).toEqual(
      expect.objectContaining({
        type: "add",
        oldLineNumber: null,
        newLineNumber: 1,
      }),
    );
  });

  it("maps unified hunks into split rows", () => {
    const diff = parseUnifiedDiff(
      [
        "@@ -1,3 +1,3 @@",
        " keep",
        "-before",
        "+after",
        "+more",
      ].join("\n"),
    );

    expect(buildSplitDiffRows(diff)).toEqual([
      { kind: "hunk", header: "@@ -1,3 +1,3 @@" },
      {
        kind: "line",
        left: expect.objectContaining({ type: "context", content: "keep" }),
        right: expect.objectContaining({ type: "context", content: "keep" }),
      },
      {
        kind: "line",
        left: expect.objectContaining({ type: "delete", content: "before" }),
        right: expect.objectContaining({ type: "add", content: "after" }),
      },
      {
        kind: "line",
        left: null,
        right: expect.objectContaining({ type: "add", content: "more" }),
      },
    ]);
  });

  it("computes diff viewer state for whitespace-only diffs", () => {
    const parsedDiff = parseUnifiedDiff("");

    expect(
      getDiffViewerState({
        selectedFilePath: "file.txt",
        loadingDiff: false,
        diffText: "",
        hideWhitespaceInDiff: true,
        parsedDiff,
      }),
    ).toBe("only-whitespace");
  });
});
