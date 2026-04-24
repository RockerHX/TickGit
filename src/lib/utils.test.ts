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

  it("formats relative dates in zh-CN", () => {
    vi.spyOn(Date, "now").mockReturnValue(
      new Date("2026-04-25T12:00:00.000Z").getTime(),
    );

    expect(formatRelativeDate("2026-04-25T11:00:00.000Z")).toBe("1小时前");
    expect(formatRelativeDate("2026-04-24T12:00:00.000Z")).toBe("昨天");
    expect(formatRelativeDate("2026-04-25T12:01:00.000Z")).toBe("1分钟后");
  });

  it("returns fallback text for invalid dates", () => {
    expect(formatRelativeDate("invalid")).toBe("Unknown time");
    expect(formatAbsoluteDate("invalid")).toBe("Unknown time");
  });

  it("formats absolute dates", () => {
    expect(formatAbsoluteDate("2026-04-25T12:00:00.000Z")).toContain("2026");
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
