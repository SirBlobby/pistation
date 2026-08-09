import type { Widget, WidgetPlacement } from "./widget.js";
import { WIDGET_GRID_COLUMNS, WIDGET_GRID_ROWS } from "./widget.js";

export type Occupancy = boolean[][];

export function buildOccupancy(widgets: Widget[], ignoreWidgetId?: string): Occupancy {
  const grid: Occupancy = Array.from({ length: WIDGET_GRID_ROWS }, () =>
    Array.from({ length: WIDGET_GRID_COLUMNS }, () => false)
  );

  for (const widget of widgets) {
    if (widget.widgetId === ignoreWidgetId) continue;

    const { column, row, columnSpan, rowSpan } = widget.placement;
    for (let y = row - 1; y < row - 1 + rowSpan; y += 1) {
      for (let x = column - 1; x < column - 1 + columnSpan; x += 1) {
        if (y >= 0 && y < WIDGET_GRID_ROWS && x >= 0 && x < WIDGET_GRID_COLUMNS) {
          grid[y][x] = true;
        }
      }
    }
  }

  return grid;
}

export function isWithinGrid(placement: WidgetPlacement): boolean {
  return (
    placement.column >= 1 &&
    placement.row >= 1 &&
    placement.columnSpan >= 1 &&
    placement.rowSpan >= 1 &&
    placement.column + placement.columnSpan - 1 <= WIDGET_GRID_COLUMNS &&
    placement.row + placement.rowSpan - 1 <= WIDGET_GRID_ROWS
  );
}

export function isAreaFree(occupancy: Occupancy, placement: WidgetPlacement): boolean {
  if (!isWithinGrid(placement)) return false;

  for (let y = placement.row - 1; y < placement.row - 1 + placement.rowSpan; y += 1) {
    for (let x = placement.column - 1; x < placement.column - 1 + placement.columnSpan; x += 1) {
      if (occupancy[y][x]) return false;
    }
  }

  return true;
}

export function clampPlacement(placement: WidgetPlacement): WidgetPlacement {
  const columnSpan = Math.min(Math.max(1, placement.columnSpan), WIDGET_GRID_COLUMNS);
  const rowSpan = Math.min(Math.max(1, placement.rowSpan), WIDGET_GRID_ROWS);

  return {
    columnSpan,
    rowSpan,
    column: Math.min(Math.max(1, placement.column), WIDGET_GRID_COLUMNS - columnSpan + 1),
    row: Math.min(Math.max(1, placement.row), WIDGET_GRID_ROWS - rowSpan + 1)
  };
}

export function findFreePlacement(
  widgets: Widget[],
  columnSpan: number,
  rowSpan: number
): WidgetPlacement | null {
  const occupancy = buildOccupancy(widgets);

  for (const span of shrinkingSpans(columnSpan, rowSpan)) {
    for (let row = 1; row <= WIDGET_GRID_ROWS - span.rowSpan + 1; row += 1) {
      for (let column = 1; column <= WIDGET_GRID_COLUMNS - span.columnSpan + 1; column += 1) {
        const candidate = { column, row, ...span };
        if (isAreaFree(occupancy, candidate)) return candidate;
      }
    }
  }

  return null;
}

function shrinkingSpans(columnSpan: number, rowSpan: number) {
  const spans: { columnSpan: number; rowSpan: number }[] = [];
  let columns = Math.min(Math.max(1, columnSpan), WIDGET_GRID_COLUMNS);
  let rows = Math.min(Math.max(1, rowSpan), WIDGET_GRID_ROWS);

  while (columns >= 1 && rows >= 1) {
    spans.push({ columnSpan: columns, rowSpan: rows });
    if (columns === 1 && rows === 1) break;
    if (columns >= rows) columns -= 1;
    else rows -= 1;
  }

  return spans;
}
