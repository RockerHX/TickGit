import { describe, expect, it } from "vitest";
import {
  DEFAULT_COMMIT_INFO_COLLAPSED,
  DEFAULT_TEXT_SELECTION_ENABLED,
  parseStoredBoolean,
} from "$lib/tickgit/preferences";

describe("preferences", () => {
  it("parses stored boolean preferences", () => {
    expect(parseStoredBoolean("true", false)).toBe(true);
    expect(parseStoredBoolean("false", true)).toBe(false);
    expect(parseStoredBoolean(null)).toBe(DEFAULT_COMMIT_INFO_COLLAPSED);
    expect(parseStoredBoolean("unknown", false)).toBe(false);
  });

  it("parses text selection preferences with the default disabled", () => {
    expect(parseStoredBoolean("true", DEFAULT_TEXT_SELECTION_ENABLED)).toBe(
      true,
    );
    expect(parseStoredBoolean("false", DEFAULT_TEXT_SELECTION_ENABLED)).toBe(
      false,
    );
    expect(parseStoredBoolean(null, DEFAULT_TEXT_SELECTION_ENABLED)).toBe(
      false,
    );
    expect(parseStoredBoolean("unknown", true)).toBe(true);
  });
});
