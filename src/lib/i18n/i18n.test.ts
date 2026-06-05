import { describe, expect, it } from "vitest";
import {
  FALLBACK_LOCALE,
  SUPPORTED_LOCALES,
  detectInitialLocale,
  isSupportedLocale,
  normalizeLocale,
  resources,
  translate,
  type TranslationKey,
} from "$lib/i18n";

describe("i18n resources", () => {
  it("keeps locale resources in sync", () => {
    const fallbackKeys = Object.keys(resources[FALLBACK_LOCALE]).sort();

    for (const locale of SUPPORTED_LOCALES) {
      expect(Object.keys(resources[locale]).sort()).toEqual(fallbackKeys);
    }
  });

  it("detects supported and normalized locales", () => {
    expect(isSupportedLocale("zh-CN")).toBe(true);
    expect(isSupportedLocale("fr-FR")).toBe(false);
    expect(normalizeLocale("zh-Hans-CN")).toBe("zh-CN");
    expect(normalizeLocale("en-GB")).toBe("en-US");
    expect(normalizeLocale("fr-FR")).toBeNull();
  });

  it("prefers stored locale before navigator languages", () => {
    expect(
      detectInitialLocale({
        storedLocale: "en-US",
        navigatorLanguages: ["zh-CN"],
      }),
    ).toBe("en-US");
    expect(
      detectInitialLocale({
        storedLocale: null,
        navigatorLanguages: ["zh-Hans-CN"],
      }),
    ).toBe("zh-CN");
    expect(
      detectInitialLocale({
        storedLocale: null,
        navigatorLanguages: ["fr-FR"],
      }),
    ).toBe("en-US");
  });

  it("translates known keys and interpolates params", () => {
    expect(
      translate("en-US", "branch.switchedMessage", { branch: "main" }),
    ).toBe("Current branch switched to main");
    expect(
      translate("zh-CN", "branch.switchedMessage", { branch: "main" }),
    ).toBe("当前已切换到 main");
  });

  it("keeps missing params as visible placeholders", () => {
    expect(translate("en-US", "push.successMessage")).toBe(
      "Pushed to {target}",
    );
  });

  it("falls back to the key for unknown keys", () => {
    expect(translate("en-US", "missing.key" as TranslationKey)).toBe(
      "missing.key",
    );
  });
});
