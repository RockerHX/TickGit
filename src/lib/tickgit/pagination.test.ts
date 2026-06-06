import { describe, expect, it } from "vitest";
import { getPaginationState, HISTORY_PAGE_SIZE } from "$lib/tickgit/pagination";

function pageLabels(totalCount: number, pageIndex: number) {
  return getPaginationState(totalCount, pageIndex).buttons.map(
    (button) => button.label,
  );
}

describe("pagination", () => {
  it("returns an empty state for empty lists", () => {
    expect(getPaginationState(0, 0)).toEqual({
      pageIndex: 0,
      pageSize: HISTORY_PAGE_SIZE,
      totalCount: 0,
      totalPages: 0,
      showingStart: 0,
      showingEnd: 0,
      canPrevious: false,
      canNext: false,
      buttons: [],
    });
  });

  it("describes the first page range and controls", () => {
    const state = getPaginationState(152, 0);

    expect(state.showingStart).toBe(1);
    expect(state.showingEnd).toBe(20);
    expect(state.totalPages).toBe(8);
    expect(state.canPrevious).toBe(false);
    expect(state.canNext).toBe(true);
    expect(pageLabels(152, 0)).toEqual(["1", "2", "3", "…", "8"]);
  });

  it("describes a middle page with ellipses on both sides", () => {
    const state = getPaginationState(152, 3);

    expect(state.showingStart).toBe(61);
    expect(state.showingEnd).toBe(80);
    expect(state.canPrevious).toBe(true);
    expect(state.canNext).toBe(true);
    expect(pageLabels(152, 3)).toEqual(["1", "…", "3", "4", "5", "…", "8"]);
  });

  it("describes the last page range and controls", () => {
    const state = getPaginationState(152, 7);

    expect(state.showingStart).toBe(141);
    expect(state.showingEnd).toBe(152);
    expect(state.canPrevious).toBe(true);
    expect(state.canNext).toBe(false);
    expect(pageLabels(152, 7)).toEqual(["1", "…", "6", "7", "8"]);
  });

  it("clamps out-of-range page indexes", () => {
    expect(getPaginationState(41, 9).pageIndex).toBe(2);
    expect(getPaginationState(41, -3).pageIndex).toBe(0);
  });
});
