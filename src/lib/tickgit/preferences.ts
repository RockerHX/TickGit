import { writable } from "svelte/store";

export const COMMIT_INFO_DEFAULT_COLLAPSED_STORAGE_KEY =
  "tickgit.commitInfoDefaultCollapsed";
export const DEFAULT_COMMIT_INFO_COLLAPSED = true;
export const TEXT_SELECTION_ENABLED_STORAGE_KEY =
  "tickgit.textSelectionEnabled";
export const DEFAULT_TEXT_SELECTION_ENABLED = false;

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

function readTextSelectionEnabled() {
  if (typeof localStorage === "undefined") {
    return DEFAULT_TEXT_SELECTION_ENABLED;
  }

  return parseStoredBoolean(
    localStorage.getItem(TEXT_SELECTION_ENABLED_STORAGE_KEY),
    DEFAULT_TEXT_SELECTION_ENABLED,
  );
}

export const commitInfoDefaultCollapsed = writable<boolean>(
  readCommitInfoDefaultCollapsed(),
);
export const textSelectionEnabled = writable<boolean>(
  readTextSelectionEnabled(),
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

export function setTextSelectionEnabled(value: boolean) {
  textSelectionEnabled.set(value);

  if (typeof localStorage !== "undefined") {
    localStorage.setItem(TEXT_SELECTION_ENABLED_STORAGE_KEY, String(value));
  }
}
