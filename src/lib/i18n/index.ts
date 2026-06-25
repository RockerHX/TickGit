import { writable } from "svelte/store";
import { FALLBACK_LOCALE, resources, type TranslationKey } from "./resources";
import {
  SUPPORTED_LOCALES,
  type Locale,
  type TranslationParams,
} from "./types";

export { FALLBACK_LOCALE, resources, SUPPORTED_LOCALES };
export type { Locale, TranslationKey, TranslationParams };

export const LOCALE_STORAGE_KEY = "tickgit.locale";
export const DEFAULT_CHINESE_LOCALE: Locale = "zh-CN";

export function isSupportedLocale(
  value: string | null | undefined,
): value is Locale {
  return SUPPORTED_LOCALES.includes(value as Locale);
}

export function normalizeLocale(
  value: string | null | undefined,
): Locale | null {
  if (!value) {
    return null;
  }

  if (isSupportedLocale(value)) {
    return value;
  }

  const normalized = value.toLowerCase();
  if (normalized === "zh" || normalized.startsWith("zh-")) {
    return DEFAULT_CHINESE_LOCALE;
  }

  if (normalized === "en" || normalized.startsWith("en-")) {
    return FALLBACK_LOCALE;
  }

  return null;
}

export function detectInitialLocale({
  storedLocale,
  navigatorLanguages,
}: {
  storedLocale?: string | null;
  navigatorLanguages?: readonly string[] | null;
} = {}): Locale {
  const stored = normalizeLocale(storedLocale);
  if (stored) {
    return stored;
  }

  for (const language of navigatorLanguages ?? []) {
    const normalized = normalizeLocale(language);
    if (normalized) {
      return normalized;
    }
  }

  return FALLBACK_LOCALE;
}

function readStoredLocale() {
  if (typeof localStorage === "undefined") {
    return null;
  }

  return localStorage.getItem(LOCALE_STORAGE_KEY);
}

function readNavigatorLanguages() {
  if (typeof navigator === "undefined") {
    return [];
  }

  return navigator.languages?.length
    ? [...navigator.languages]
    : navigator.language
      ? [navigator.language]
      : [];
}

function syncDocumentLanguage(nextLocale: Locale) {
  if (typeof document === "undefined") {
    return;
  }

  document.documentElement.lang = nextLocale;
}

const initialLocale = detectInitialLocale({
  storedLocale: readStoredLocale(),
  navigatorLanguages: readNavigatorLanguages(),
});

syncDocumentLanguage(initialLocale);

export const locale = writable<Locale>(initialLocale);

export function setLocale(nextLocale: Locale) {
  locale.set(nextLocale);

  if (typeof localStorage !== "undefined") {
    localStorage.setItem(LOCALE_STORAGE_KEY, nextLocale);
  }

  syncDocumentLanguage(nextLocale);
}

const ERROR_CODE_KEYS: Partial<Record<string, TranslationKey>> = {
  repository_missing: "repository.status.missingMessage",
  repository_exists: "error.repositoryExists",
  repository_not_found: "error.repositoryNotFound",
  config_dir_unavailable: "error.configDirUnavailable",
  config_dir_create_failed: "error.configDirCreateFailed",
  store_read_failed: "error.storeReadFailed",
  store_parse_failed: "error.storeParseFailed",
  store_serialize_failed: "error.storeSerializeFailed",
  store_write_failed: "error.storeWriteFailed",
  window_resize_failed: "error.windowResizeFailed",
  monitor_lookup_failed: "error.monitorLookupFailed",
  monitor_unavailable: "error.monitorUnavailable",
  window_size_invalid: "error.windowSizeInvalid",
  window_not_found: "error.windowNotFound",
  git_unavailable: "error.gitUnavailable",
  git_command_failed: "error.gitCommandFailed",
  external_tool_unavailable: "error.externalToolUnavailable",
  invalid_repository: "error.invalidRepository",
  invalid_branch: "error.invalidBranch",
  checkout_blocked_by_local_changes: "error.checkoutBlockedByLocalChanges",
  checkout_blocked_by_untracked_files: "error.checkoutBlockedByUntrackedFiles",
  detached_head: "push.detachedHead",
  branch_mismatch: "push.branchMismatch",
  missing_origin: "error.missingOrigin",
  missing_upstream: "push.missingUpstream",
  non_origin_upstream: "error.nonOriginUpstream",
  behind_remote: "branch.behindRemote",
  push_unavailable: "branch.currentPushUnavailable",
  push_busy: "push.busy",
  step_push_busy: "push.stepBusy",
  push_to_commit_busy: "push.toCommitBusy",
  empty_hashes: "push.emptyHashes",
  invalid_hash: "push.invalidHash",
  unsafe_push_target: "push.unsafeTarget",
};

const BRANCH_DISABLED_REASON_KEYS: Partial<Record<string, TranslationKey>> = {
  detached_head: "branch.detachedPushDisabled",
  missing_origin: "branch.missingOriginPushDisabled",
  missing_upstream: "branch.missingUpstreamPushDisabled",
  non_origin_upstream: "branch.nonOriginUpstreamPushDisabled",
  behind_remote: "branch.behindRemote",
  push_unavailable: "branch.currentPushUnavailable",
};

export function errorCodeTranslationKey(code: string | null | undefined) {
  return code ? (ERROR_CODE_KEYS[code] ?? null) : null;
}

export function branchDisabledReasonTranslationKey(
  code: string | null | undefined,
) {
  return code ? (BRANCH_DISABLED_REASON_KEYS[code] ?? null) : null;
}

export function translateErrorCode(
  locale: Locale,
  code: string | null | undefined,
  fallback: string | null | undefined = null,
) {
  const key = errorCodeTranslationKey(code);
  return key
    ? translate(locale, key)
    : (fallback ?? translate(locale, "common.unknownError"));
}

export function translateBranchDisabledReason(
  locale: Locale,
  code: string | null | undefined,
  fallback: string | null | undefined = null,
) {
  const key = branchDisabledReasonTranslationKey(code);
  return key
    ? translate(locale, key)
    : (fallback ?? translate(locale, "branch.pushDisabledFallback"));
}

export function translate(
  locale: Locale,
  key: TranslationKey,
  params: TranslationParams = {},
) {
  const localeResources = resources[locale] ?? resources[FALLBACK_LOCALE];
  const template =
    localeResources[key] ?? resources[FALLBACK_LOCALE][key] ?? key;

  return template.replace(/\{(\w+)\}/g, (match, name) => {
    const value = params[name];
    return value === undefined ? match : String(value);
  });
}

export const t = translate;
