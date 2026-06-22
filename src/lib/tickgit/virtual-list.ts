export type VirtualWindowInput = {
  totalRows: number;
  scrollTop: number;
  viewportHeight: number;
  rowHeight: number;
  overscanRows: number;
};

export type VirtualWindow = {
  startIndex: number;
  endIndex: number;
  topSpacerHeight: number;
  bottomSpacerHeight: number;
};

function normalizeNonNegativeInteger(value: number) {
  return Math.max(0, Math.floor(Number.isFinite(value) ? value : 0));
}

export function getVirtualWindow(input: VirtualWindowInput): VirtualWindow {
  const totalRows = normalizeNonNegativeInteger(input.totalRows);
  const rowHeight = Math.max(1, Math.floor(input.rowHeight) || 1);
  const overscanRows = normalizeNonNegativeInteger(input.overscanRows);
  const scrollTop = Math.max(
    0,
    Number.isFinite(input.scrollTop) ? input.scrollTop : 0,
  );
  const viewportHeight = Math.max(
    0,
    Number.isFinite(input.viewportHeight) ? input.viewportHeight : 0,
  );

  if (totalRows === 0) {
    return {
      startIndex: 0,
      endIndex: 0,
      topSpacerHeight: 0,
      bottomSpacerHeight: 0,
    };
  }

  const firstVisibleIndex = Math.min(
    totalRows - 1,
    Math.floor(scrollTop / rowHeight),
  );
  const visibleRows = Math.max(1, Math.ceil(viewportHeight / rowHeight));
  const startIndex = Math.max(0, firstVisibleIndex - overscanRows);
  const endIndex = Math.min(
    totalRows,
    startIndex + visibleRows + overscanRows * 2,
  );

  return {
    startIndex,
    endIndex,
    topSpacerHeight: startIndex * rowHeight,
    bottomSpacerHeight: Math.max(0, (totalRows - endIndex) * rowHeight),
  };
}
