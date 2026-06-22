import { describe, expect, it } from "vitest";
import { processDiffWorkerRequest } from "$lib/tickgit/diff-worker";

describe("diff worker", () => {
  it("parses and highlights unified diff lines", () => {
    const response = processDiffWorkerRequest({
      id: 7,
      diffText: ["@@ -1,1 +1,1 @@", "-<old>", "+<new>"].join("\n"),
      filePath: null,
      mode: "unified",
    });

    expect(response.id).toBe(7);
    expect(response.error).toBeUndefined();
    expect(response.parsedDiff.hunks[0]?.lines).toEqual([
      expect.objectContaining({ type: "delete", html: "&lt;old&gt;" }),
      expect.objectContaining({ type: "add", html: "&lt;new&gt;" }),
    ]);
    expect(response.splitRows).toEqual([]);
  });

  it("builds highlighted split rows for split mode", () => {
    const response = processDiffWorkerRequest({
      id: 8,
      diffText: ["@@ -1,1 +1,1 @@", "-old", "+new"].join("\n"),
      filePath: null,
      mode: "split",
    });

    expect(response.splitRows).toEqual([
      { kind: "hunk", header: "@@ -1,1 +1,1 @@", hunkIndex: 0 },
      {
        kind: "line",
        left: expect.objectContaining({ type: "delete", html: "old" }),
        right: expect.objectContaining({ type: "add", html: "new" }),
      },
    ]);
  });
});
