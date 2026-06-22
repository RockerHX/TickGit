import {
  getSplitDiffRowsForMode,
  parseUnifiedDiff,
  type DiffHunk,
  type DiffLine,
  type ParsedTextDiff,
  type SplitDiffRow,
} from "$lib/tickgit/diff";
import { highlightDiffContent } from "$lib/tickgit/diff-highlight";

export type HighlightedDiffLine = DiffLine & {
  html: string;
};

type HighlightedDiffHunk = Omit<DiffHunk, "lines"> & {
  lines: HighlightedDiffLine[];
};

export type HighlightedParsedTextDiff = Omit<ParsedTextDiff, "hunks"> & {
  hunks: HighlightedDiffHunk[];
};

export type HighlightedSplitDiffRow =
  | {
      kind: "hunk";
      header: string;
      hunkIndex: number;
    }
  | {
      kind: "line";
      left: HighlightedDiffLine | null;
      right: HighlightedDiffLine | null;
    };

export type DiffWorkerRequest = {
  id: number;
  diffText: string;
  filePath: string | null;
  mode: "unified" | "split";
};

export type DiffWorkerResponse = {
  id: number;
  parsedDiff: HighlightedParsedTextDiff;
  splitRows: HighlightedSplitDiffRow[];
  error?: string;
};

function highlightLine(
  line: DiffLine,
  filePath: string | null,
): HighlightedDiffLine {
  return {
    ...line,
    html: highlightDiffContent(line.content || " ", filePath),
  };
}

function highlightParsedDiff(
  parsedDiff: ParsedTextDiff,
  filePath: string | null,
): HighlightedParsedTextDiff {
  return {
    ...parsedDiff,
    hunks: parsedDiff.hunks.map((hunk) => ({
      ...hunk,
      lines: hunk.lines.map((line) => highlightLine(line, filePath)),
    })),
  };
}

function emptyHighlightedParsedDiff(
  parseError = false,
): HighlightedParsedTextDiff {
  return {
    hunks: [],
    isEmpty: true,
    maxLineNumberWidth: 1,
    parseError,
  };
}

function highlightSplitRows(
  rows: SplitDiffRow[],
  highlighted: HighlightedParsedTextDiff,
): HighlightedSplitDiffRow[] {
  const linesByOriginalLine = new Map<number, HighlightedDiffLine>();
  for (const hunk of highlighted.hunks) {
    for (const line of hunk.lines) {
      linesByOriginalLine.set(line.originalLineNumber, line);
    }
  }

  return rows.map((row) => {
    if (row.kind === "hunk") {
      return row;
    }

    return {
      kind: "line",
      left: row.left
        ? (linesByOriginalLine.get(row.left.originalLineNumber) ?? null)
        : null,
      right: row.right
        ? (linesByOriginalLine.get(row.right.originalLineNumber) ?? null)
        : null,
    };
  });
}

export function processDiffWorkerRequest(
  request: DiffWorkerRequest,
): DiffWorkerResponse {
  try {
    const { id, diffText, filePath, mode } = request;
    const parsedDiff = parseUnifiedDiff(diffText);
    const highlighted = highlightParsedDiff(parsedDiff, filePath);
    const splitRows = highlightSplitRows(
      getSplitDiffRowsForMode(parsedDiff, mode),
      highlighted,
    );

    return {
      id,
      parsedDiff: highlighted,
      splitRows,
    };
  } catch (error) {
    return {
      id: request.id,
      parsedDiff: emptyHighlightedParsedDiff(true),
      splitRows: [],
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

if (typeof self !== "undefined") {
  self.onmessage = (event: MessageEvent<DiffWorkerRequest>) => {
    self.postMessage(processDiffWorkerRequest(event.data));
  };
}
