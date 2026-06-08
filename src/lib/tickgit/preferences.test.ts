import { describe, expect, it } from "vitest";
import {
  DEFAULT_COMMIT_INFO_COLLAPSED,
  parseStoredBoolean,
} from "$lib/tickgit/preferences";

describe("preferences", () => {
  it("parses stored boolean preferences", () => {
    expect(parseStoredBoolean("true", false)).toBe(true);
    expect(parseStoredBoolean("false", true)).toBe(false);
    expect(parseStoredBoolean(null)).toBe(DEFAULT_COMMIT_INFO_COLLAPSED);
    expect(parseStoredBoolean("unknown", false)).toBe(false);
  });
});
