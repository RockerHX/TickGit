import { writable } from "svelte/store";
import { FALLBACK_LOCALE, resources, type TranslationKey } from "./resources";
import { SUPPORTED_LOCALES, type Locale, type TranslationParams } from "./types";

export { FALLBACK_LOCALE, resources, SUPPORTED_LOCALES };
export type { Locale, TranslationKey, TranslationParams };

export const LOCALE_STORAGE_KEY = "tickgit.locale";
export const DEFAULT_CHINESE_LOCALE: Locale = "zh-CN";

export function isSupportedLocale(value: string | null | undefined): value is Locale {
  return SUPPORTED_LOCALES.includes(value as Locale);
}

export function normalizeLocale(value: string | null | undefined): Locale | null {
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

export function translate(
  locale: Locale,
  key: TranslationKey,
  params: TranslationParams = {},
) {
  const template = resources[locale][key] ?? resources[FALLBACK_LOCALE][key] ?? key;

  return template.replace(/\{(\w+)\}/g, (match, name) => {
    const value = params[name];
    return value === undefined ? match : String(value);
  });
}

export const t = translate;
