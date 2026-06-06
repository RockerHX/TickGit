export const HISTORY_PAGE_SIZE = 20;

export type PageButtonModel =
  | {
      kind: "page";
      label: string;
      pageIndex: number;
      active: boolean;
    }
  | {
      kind: "ellipsis";
      label: "…";
      key: string;
    };

export type PaginationState = {
  pageIndex: number;
  pageSize: number;
  totalCount: number;
  totalPages: number;
  showingStart: number;
  showingEnd: number;
  canPrevious: boolean;
  canNext: boolean;
  buttons: PageButtonModel[];
};

function normalizePageSize(pageSize: number) {
  return Math.max(1, Math.floor(pageSize) || HISTORY_PAGE_SIZE);
}

function clampPageIndex(pageIndex: number, totalPages: number) {
  if (totalPages <= 0) {
    return 0;
  }

  return Math.min(
    Math.max(Math.floor(Number.isFinite(pageIndex) ? pageIndex : 0), 0),
    totalPages - 1,
  );
}

function visiblePageIndexes(pageIndex: number, totalPages: number) {
  if (totalPages <= 5) {
    return Array.from({ length: totalPages }, (_, index) => index);
  }

  const lastPageIndex = totalPages - 1;
  const visible = new Set<number>([0, lastPageIndex]);

  if (pageIndex <= 1) {
    visible.add(1);
    visible.add(2);
  } else if (pageIndex >= lastPageIndex - 1) {
    visible.add(lastPageIndex - 2);
    visible.add(lastPageIndex - 1);
  } else {
    visible.add(pageIndex - 1);
    visible.add(pageIndex);
    visible.add(pageIndex + 1);
  }

  return Array.from(visible).sort((left, right) => left - right);
}

function buildPageButtons(pageIndex: number, totalPages: number) {
  const pageIndexes = visiblePageIndexes(pageIndex, totalPages);
  const buttons: PageButtonModel[] = [];

  for (const currentIndex of pageIndexes) {
    const previous = buttons.at(-1);
    if (previous?.kind === "page" && currentIndex - previous.pageIndex > 1) {
      buttons.push({
        kind: "ellipsis",
        label: "…",
        key: `${previous.pageIndex}-${currentIndex}`,
      });
    }

    buttons.push({
      kind: "page",
      label: String(currentIndex + 1),
      pageIndex: currentIndex,
      active: currentIndex === pageIndex,
    });
  }

  return buttons;
}

export function getPaginationState(
  totalCount: number,
  requestedPageIndex: number,
  requestedPageSize = HISTORY_PAGE_SIZE,
): PaginationState {
  const pageSize = normalizePageSize(requestedPageSize);
  const normalizedTotalCount = Math.max(0, Math.floor(totalCount) || 0);
  const totalPages =
    normalizedTotalCount === 0 ? 0 : Math.ceil(normalizedTotalCount / pageSize);
  const pageIndex = clampPageIndex(requestedPageIndex, totalPages);
  const showingStart =
    normalizedTotalCount === 0 ? 0 : pageIndex * pageSize + 1;
  const showingEnd = Math.min(normalizedTotalCount, (pageIndex + 1) * pageSize);

  return {
    pageIndex,
    pageSize,
    totalCount: normalizedTotalCount,
    totalPages,
    showingStart,
    showingEnd,
    canPrevious: pageIndex > 0,
    canNext: totalPages > 0 && pageIndex < totalPages - 1,
    buttons: buildPageButtons(pageIndex, totalPages),
  };
}
