import { writable } from "svelte/store";

export const COMMIT_INFO_DEFAULT_COLLAPSED_STORAGE_KEY =
  "tickgit.commitInfoDefaultCollapsed";
export const DEFAULT_COMMIT_INFO_COLLAPSED = true;

export function parseStoredBoolean(
  value: string | null | undefined,
  fallback = DEFAULT_COMMIT_INFO_COLLAPSED,
) {
  if (value === "true") {
    return true;
  }

  if (value === "false") {
    return false;
  }

  return fallback;
}

function readCommitInfoDefaultCollapsed() {
  if (typeof localStorage === "undefined") {
    return DEFAULT_COMMIT_INFO_COLLAPSED;
  }

  return parseStoredBoolean(
    localStorage.getItem(COMMIT_INFO_DEFAULT_COLLAPSED_STORAGE_KEY),
  );
}

export const commitInfoDefaultCollapsed = writable<boolean>(
  readCommitInfoDefaultCollapsed(),
);

export function setCommitInfoDefaultCollapsed(value: boolean) {
  commitInfoDefaultCollapsed.set(value);

  if (typeof localStorage !== "undefined") {
    localStorage.setItem(
      COMMIT_INFO_DEFAULT_COLLAPSED_STORAGE_KEY,
      String(value),
    );
  }
}
