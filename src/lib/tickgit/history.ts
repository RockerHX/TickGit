import type { CommitFileChange, CommitHistoryFilters } from "$lib/types";

export const EMPTY_HISTORY_FILTERS: CommitHistoryFilters = {
  query: "",
  author: "",
  filePath: "",
  message: "",
};

export function normalizeHistoryFilters(
  filters: CommitHistoryFilters | null | undefined,
): CommitHistoryFilters {
  return {
    query: filters?.query?.trim() ?? "",
    author: filters?.author?.trim() ?? "",
    filePath: filters?.filePath?.trim() ?? "",
    message: filters?.message?.trim() ?? "",
  };
}

export function getActiveHistoryFilterCount(
  filters: CommitHistoryFilters | null | undefined,
) {
  const normalized = normalizeHistoryFilters(filters);

  return [
    normalized.query,
    normalized.author,
    normalized.filePath,
    normalized.message,
  ].filter(Boolean).length;
}

export function hasActiveHistoryFilters(
  filters: CommitHistoryFilters | null | undefined,
) {
  return getActiveHistoryFilterCount(filters) > 0;
}

export function historyFiltersEqual(
  left: CommitHistoryFilters | null | undefined,
  right: CommitHistoryFilters | null | undefined,
) {
  const normalizedLeft = normalizeHistoryFilters(left);
  const normalizedRight = normalizeHistoryFilters(right);

  return (
    normalizedLeft.query === normalizedRight.query &&
    normalizedLeft.author === normalizedRight.author &&
    normalizedLeft.filePath === normalizedRight.filePath &&
    normalizedLeft.message === normalizedRight.message
  );
}

function containsPathFragment(
  value: string | null | undefined,
  filter: string,
) {
  return value?.toLocaleLowerCase().includes(filter) ?? false;
}

export function commitFileMatchesPathFilter(
  file: CommitFileChange,
  filePathFilter: string | null | undefined,
) {
  const normalizedFilter = filePathFilter?.trim().toLocaleLowerCase() ?? "";

  if (!normalizedFilter) {
    return true;
  }

  return (
    containsPathFragment(file.path, normalizedFilter) ||
    containsPathFragment(file.previousPath, normalizedFilter) ||
    containsPathFragment(file.displayPath, normalizedFilter)
  );
}

export function pickCommitFileForPathFilter(
  files: CommitFileChange[],
  filePathFilter: string | null | undefined,
) {
  return (
    files.find((file) => commitFileMatchesPathFilter(file, filePathFilter)) ??
    files[0] ??
    null
  );
}
