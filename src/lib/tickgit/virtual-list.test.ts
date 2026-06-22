import { describe, expect, it } from "vitest";
import { getVirtualWindow } from "$lib/tickgit/virtual-list";

describe("virtual list window", () => {
  it("returns an empty window for empty lists", () => {
    expect(
      getVirtualWindow({
        totalRows: 0,
        scrollTop: 120,
        viewportHeight: 200,
        rowHeight: 20,
        overscanRows: 5,
      }),
    ).toEqual({
      startIndex: 0,
      endIndex: 0,
      topSpacerHeight: 0,
      bottomSpacerHeight: 0,
    });
  });

  it("includes overscan before and after the visible range", () => {
    expect(
      getVirtualWindow({
        totalRows: 100,
        scrollTop: 200,
        viewportHeight: 100,
        rowHeight: 20,
        overscanRows: 2,
      }),
    ).toEqual({
      startIndex: 8,
      endIndex: 17,
      topSpacerHeight: 160,
      bottomSpacerHeight: 1660,
    });
  });

  it("clamps the window near the top of the list", () => {
    expect(
      getVirtualWindow({
        totalRows: 10,
        scrollTop: 0,
        viewportHeight: 60,
        rowHeight: 20,
        overscanRows: 3,
      }),
    ).toEqual({
      startIndex: 0,
      endIndex: 9,
      topSpacerHeight: 0,
      bottomSpacerHeight: 20,
    });
  });

  it("clamps the window near the bottom of the list", () => {
    expect(
      getVirtualWindow({
        totalRows: 10,
        scrollTop: 190,
        viewportHeight: 60,
        rowHeight: 20,
        overscanRows: 2,
      }),
    ).toEqual({
      startIndex: 7,
      endIndex: 10,
      topSpacerHeight: 140,
      bottomSpacerHeight: 0,
    });
  });

  it("keeps large file lists bounded to the visible window", () => {
    const window = getVirtualWindow({
      totalRows: 1000,
      scrollTop: 4200,
      viewportHeight: 420,
      rowHeight: 42,
      overscanRows: 12,
    });

    expect(window.endIndex - window.startIndex).toBeLessThan(1000);
    expect(window.endIndex - window.startIndex).toBeLessThanOrEqual(34);
    expect(window.startIndex).toBe(88);
  });

  it("normalizes invalid numeric input", () => {
    expect(
      getVirtualWindow({
        totalRows: Number.NaN,
        scrollTop: Number.NaN,
        viewportHeight: Number.NaN,
        rowHeight: 0,
        overscanRows: -1,
      }),
    ).toEqual({
      startIndex: 0,
      endIndex: 0,
      topSpacerHeight: 0,
      bottomSpacerHeight: 0,
    });
  });
});
