import { afterEach, describe, expect, it, vi } from "vitest";
import {
  diffLineClass,
  formatAbsoluteDate,
  formatRelativeDate,
  statusTone,
} from "$lib/utils";

describe("utils", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("formats relative dates with the requested locale", () => {
    vi.spyOn(Date, "now").mockReturnValue(
      new Date("2026-04-25T12:00:00.000Z").getTime(),
    );

    expect(formatRelativeDate("2026-04-25T11:00:00.000Z", "en-US")).toBe(
      "1 hour ago",
    );
    expect(formatRelativeDate("2026-04-24T12:00:00.000Z", "en-US")).toBe(
      "yesterday",
    );
    expect(formatRelativeDate("2026-04-25T12:01:00.000Z", "en-US")).toBe(
      "in 1 minute",
    );
    expect(formatRelativeDate("2026-04-25T11:00:00.000Z", "zh-CN")).toBe(
      "1小时前",
    );
  });

  it("returns fallback text for invalid dates", () => {
    expect(formatRelativeDate("invalid", "en-US")).toBe("Unknown time");
    expect(formatAbsoluteDate("invalid", "en-US")).toBe("Unknown time");
    expect(formatRelativeDate("invalid", "zh-CN")).toBe("未知时间");
    expect(formatAbsoluteDate("invalid", "zh-CN")).toBe("未知时间");
  });

  it("formats absolute dates", () => {
    expect(formatAbsoluteDate("2026-04-25T12:00:00.000Z", "en-US")).toContain(
      "2026",
    );
    expect(formatAbsoluteDate("2026-04-25T12:00:00.000Z", "zh-CN")).toContain(
      "2026",
    );
  });

  it("maps diff lines to style tokens", () => {
    expect(diffLineClass("+added")).toContain("emerald");
    expect(diffLineClass("-removed")).toContain("rose");
    expect(diffLineClass("@@ section")).toContain("sky");
    expect(diffLineClass(" context")).toContain("slate");
    expect(diffLineClass("+++ b/file.txt")).toContain("slate");
    expect(diffLineClass("--- a/file.txt")).toContain("slate");
  });

  it("maps file status to tone classes", () => {
    expect(statusTone("A")).toContain("emerald");
    expect(statusTone("M")).toContain("amber");
    expect(statusTone("D")).toContain("rose");
    expect(statusTone("R100")).toContain("sky");
    expect(statusTone("C100")).toContain("slate");
  });
});
