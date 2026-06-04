export type DiffLineType = "context" | "add" | "delete" | "hunk";

export type DiffLine = {
  raw: string;
  content: string;
  type: Exclude<DiffLineType, "hunk">;
  originalLineNumber: number;
  oldLineNumber: number | null;
  newLineNumber: number | null;
  noTrailingNewLine: boolean;
};

export type DiffHunk = {
  header: string;
  oldStartLine: number;
  oldLineCount: number;
  newStartLine: number;
  newLineCount: number;
  lines: DiffLine[];
};

export type ParsedTextDiff = {
  hunks: DiffHunk[];
  isEmpty: boolean;
  maxLineNumberWidth: number;
  parseError: boolean;
};

export type SplitDiffRow =
  | {
      kind: "hunk";
      header: string;
    }
  | {
      kind: "line";
      left: DiffLine | null;
      right: DiffLine | null;
    };

export type DiffViewerState =
  | "loading"
  | "no-file"
  | "binary"
  | "image"
  | "too-large"
  | "only-whitespace"
  | "no-content"
  | "parse-error"
  | "ready";

const HUNK_HEADER_RE = /^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@/;

function createParsedDiff(
  hunks: DiffHunk[],
  parseError = false,
): ParsedTextDiff {
  const maxLineNumber = hunks.reduce((currentMax, hunk) => {
    const lineMax = hunk.lines.reduce((lineCurrentMax, line) => {
      return Math.max(
        lineCurrentMax,
        line.oldLineNumber ?? 0,
        line.newLineNumber ?? 0,
      );
    }, 0);

    return Math.max(currentMax, hunk.oldStartLine, hunk.newStartLine, lineMax);
  }, 0);

  return {
    hunks,
    isEmpty: hunks.every((hunk) => hunk.lines.length === 0),
    maxLineNumberWidth: String(maxLineNumber).length,
    parseError,
  };
}

// 统一先解析 unified diff，再从结构化模型派生 split 视图；
// 这样可以复用同一份行号与 hunk 语义，避免两套解析逻辑长期漂移。
export function parseUnifiedDiff(diffText: string): ParsedTextDiff {
  if (!diffText.trim()) {
    return createParsedDiff([]);
  }

  try {
    const hunks: DiffHunk[] = [];
    const lines = diffText.split("\n");
    let currentHunk: DiffHunk | null = null;
    let oldLineNumber = 0;
    let newLineNumber = 0;
    let originalLineNumber = 0;
    let sawUnsupportedContent = false;

    for (const line of lines) {
      originalLineNumber += 1;

      if (!line && originalLineNumber === lines.length) {
        continue;
      }

      const hunkMatch = HUNK_HEADER_RE.exec(line);
      if (hunkMatch) {
        currentHunk = {
          header: line,
          oldStartLine: Number(hunkMatch[1]),
          oldLineCount: Number(hunkMatch[2] ?? 1),
          newStartLine: Number(hunkMatch[3]),
          newLineCount: Number(hunkMatch[4] ?? 1),
          lines: [],
        };
        hunks.push(currentHunk);
        oldLineNumber = currentHunk.oldStartLine;
        newLineNumber = currentHunk.newStartLine;
        continue;
      }

      if (!currentHunk) {
        if (line.trim()) {
          sawUnsupportedContent = true;
        }
        continue;
      }

      if (line.startsWith("\\ ")) {
        const previousLine = currentHunk.lines.at(-1);
        if (previousLine) {
          previousLine.noTrailingNewLine = true;
        }
        continue;
      }

      const prefix = line[0];
      const content = line.slice(1);

      if (prefix === " ") {
        currentHunk.lines.push({
          raw: line,
          content,
          type: "context",
          originalLineNumber,
          oldLineNumber,
          newLineNumber,
          noTrailingNewLine: false,
        });
        oldLineNumber += 1;
        newLineNumber += 1;
        continue;
      }

      if (prefix === "-") {
        currentHunk.lines.push({
          raw: line,
          content,
          type: "delete",
          originalLineNumber,
          oldLineNumber,
          newLineNumber: null,
          noTrailingNewLine: false,
        });
        oldLineNumber += 1;
        continue;
      }

      if (prefix === "+") {
        currentHunk.lines.push({
          raw: line,
          content,
          type: "add",
          originalLineNumber,
          oldLineNumber: null,
          newLineNumber,
          noTrailingNewLine: false,
        });
        newLineNumber += 1;
        continue;
      }

      if (line.trim()) {
        sawUnsupportedContent = true;
      }
    }

    const parseError = sawUnsupportedContent && hunks.length === 0;
    return createParsedDiff(hunks, parseError);
  } catch {
    return createParsedDiff([], true);
  }
}

function flushSplitBuffer(
  rows: SplitDiffRow[],
  deletedLines: DiffLine[],
  addedLines: DiffLine[],
) {
  const length = Math.max(deletedLines.length, addedLines.length);

  for (let index = 0; index < length; index += 1) {
    rows.push({
      kind: "line",
      left: deletedLines[index] ?? null,
      right: addedLines[index] ?? null,
    });
  }

  deletedLines.length = 0;
  addedLines.length = 0;
}

export function buildSplitDiffRows(parsedDiff: ParsedTextDiff): SplitDiffRow[] {
  const rows: SplitDiffRow[] = [];

  for (const hunk of parsedDiff.hunks) {
    rows.push({ kind: "hunk", header: hunk.header });

    const deletedLines: DiffLine[] = [];
    const addedLines: DiffLine[] = [];

    for (const line of hunk.lines) {
      if (line.type === "delete") {
        deletedLines.push(line);
        continue;
      }

      if (line.type === "add") {
        addedLines.push(line);
        continue;
      }

      flushSplitBuffer(rows, deletedLines, addedLines);
      rows.push({
        kind: "line",
        left: line,
        right: line,
      });
    }

    flushSplitBuffer(rows, deletedLines, addedLines);
  }

  return rows;
}

export function getSplitDiffRowsForMode(
  parsedDiff: ParsedTextDiff,
  mode: "unified" | "split",
): SplitDiffRow[] {
  return mode === "split" ? buildSplitDiffRows(parsedDiff) : [];
}

export function getDiffViewerState(input: {
  selectedFilePath: string | null;
  loadingDiff: boolean;
  diffText: string;
  isBinary?: boolean;
  isImage?: boolean;
  isTooLarge?: boolean;
  hideWhitespaceInDiff: boolean;
  parsedDiff: ParsedTextDiff;
}): DiffViewerState {
  if (input.loadingDiff) {
    return "loading";
  }

  if (!input.selectedFilePath) {
    return "no-file";
  }

  if (input.isImage) {
    return "image";
  }

  if (input.isBinary) {
    return "binary";
  }

  if (input.isTooLarge) {
    return "too-large";
  }

  if (!input.diffText.trim()) {
    return input.hideWhitespaceInDiff ? "only-whitespace" : "no-content";
  }

  if (input.parsedDiff.parseError) {
    return "parse-error";
  }

  if (input.parsedDiff.isEmpty) {
    return input.hideWhitespaceInDiff ? "only-whitespace" : "no-content";
  }

  return "ready";
}
