import { FALLBACK_LOCALE, resources, type TranslationKey } from "./resources";
import { SUPPORTED_LOCALES, type Locale, type TranslationParams } from "./types";

export { FALLBACK_LOCALE, resources, SUPPORTED_LOCALES };
export type { Locale, TranslationKey, TranslationParams };

export function isSupportedLocale(value: string | null | undefined): value is Locale {
  return SUPPORTED_LOCALES.includes(value as Locale);
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
