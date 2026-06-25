import { describe, expect, it } from "vitest";
import {
  FALLBACK_LOCALE,
  SUPPORTED_LOCALES,
  detectInitialLocale,
  isSupportedLocale,
  normalizeLocale,
  resources,
  translate,
  translateErrorCode,
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

  it("translates checkout blocked error codes", () => {
    expect(
      translateErrorCode("en-US", "checkout_blocked_by_local_changes"),
    ).toBe(
      "Local uncommitted changes would be overwritten by the target branch. Git blocked the switch. Commit, stage, stash, or discard those changes and try again.",
    );
    expect(
      translateErrorCode("zh-CN", "checkout_blocked_by_untracked_files"),
    ).toBe(
      "未跟踪文件会被目标分支覆盖，Git 已阻止切换。请先移动、删除、加入版本控制或 stash 这些文件后重试。",
    );
  });
});
