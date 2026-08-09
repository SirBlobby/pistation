<script lang="ts">
  import type { KioskLayout, Widget, WidgetPlacement } from "@pistation/shared-types";
  import {
    buildOccupancy,
    clampPlacement,
    isAreaFree,
    WIDGET_GRID_COLUMNS,
    WIDGET_GRID_ROWS
  } from "@pistation/shared-types";
  import { WidgetSurface } from "@pistation/ui";

  let {
    layout,
    pin = null,
    joinUrl = "",
    mediaBaseUrl = "",
    selectedWidgetId = null,
    onSelect,
    onWidgetsChange
  }: {
    layout: KioskLayout;
    pin?: string | null;
    joinUrl?: string;
    mediaBaseUrl?: string;
    selectedWidgetId?: string | null;
    onSelect: (widgetId: string) => void;
    onWidgetsChange: (widgets: Widget[]) => void;
  } = $props();

  const GRID_PADDING_RATIO = 0.016;
  const GRID_GAP_RATIO = 0.012;

  interface DragState {
    widgetId: string;
    mode: "move" | "resize";
    pointerX: number;
    pointerY: number;
    origin: WidgetPlacement;
  }

  let surfaceWidth = $state(0);
  let surfaceHeight = $state(0);
  let drag = $state<DragState | null>(null);
  let draft = $state<WidgetPlacement | null>(null);
  let isDraftValid = $state(true);

  const visibleWidgets = $derived(layout.widgets.filter((widget) => widget.enabled));

  const stepX = $derived.by(() => {
    const padding = surfaceWidth * GRID_PADDING_RATIO;
    const gap = surfaceWidth * GRID_GAP_RATIO;
    const content = surfaceWidth - padding * 2;
    const cell = (content - gap * (WIDGET_GRID_COLUMNS - 1)) / WIDGET_GRID_COLUMNS;
    return cell + gap;
  });

  const stepY = $derived.by(() => {
    const padding = surfaceWidth * GRID_PADDING_RATIO;
    const gap = surfaceHeight * GRID_GAP_RATIO;
    const content = surfaceHeight - padding * 2;
    const cell = (content - gap * (WIDGET_GRID_ROWS - 1)) / WIDGET_GRID_ROWS;
    return cell + gap;
  });

  function placementFor(widget: Widget): WidgetPlacement {
    return drag?.widgetId === widget.widgetId && draft ? draft : widget.placement;
  }

  function beginDrag(event: PointerEvent, widget: Widget, mode: "move" | "resize") {
    event.preventDefault();
    event.stopPropagation();

    onSelect(widget.widgetId);
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);

    drag = {
      widgetId: widget.widgetId,
      mode,
      pointerX: event.clientX,
      pointerY: event.clientY,
      origin: { ...widget.placement }
    };
    draft = { ...widget.placement };
    isDraftValid = true;
  }

  function updateDrag(event: PointerEvent) {
    if (!drag || stepX <= 0 || stepY <= 0) return;

    const deltaColumns = Math.round((event.clientX - drag.pointerX) / stepX);
    const deltaRows = Math.round((event.clientY - drag.pointerY) / stepY);

    const candidate =
      drag.mode === "move"
        ? clampPlacement({
            ...drag.origin,
            column: drag.origin.column + deltaColumns,
            row: drag.origin.row + deltaRows
          })
        : clampPlacement({
            ...drag.origin,
            columnSpan: Math.min(
              Math.max(1, drag.origin.columnSpan + deltaColumns),
              WIDGET_GRID_COLUMNS - drag.origin.column + 1
            ),
            rowSpan: Math.min(
              Math.max(1, drag.origin.rowSpan + deltaRows),
              WIDGET_GRID_ROWS - drag.origin.row + 1
            )
          });

    draft = candidate;
    isDraftValid = isAreaFree(buildOccupancy(layout.widgets, drag.widgetId), candidate);
  }

  function endDrag() {
    if (drag && draft && isDraftValid) {
      commitPlacement(drag.widgetId, draft);
    }
    drag = null;
    draft = null;
    isDraftValid = true;
  }

  function commitPlacement(widgetId: string, placement: WidgetPlacement) {
    onWidgetsChange(
      layout.widgets.map((widget) =>
        widget.widgetId === widgetId ? { ...widget, placement } : widget
      )
    );
  }

  function nudge(event: KeyboardEvent, widget: Widget) {
    const directions: Record<string, [number, number]> = {
      ArrowLeft: [-1, 0],
      ArrowRight: [1, 0],
      ArrowUp: [0, -1],
      ArrowDown: [0, 1]
    };

    const direction = directions[event.key];
    if (!direction) return;

    event.preventDefault();
    const [deltaColumns, deltaRows] = direction;

    const candidate = clampPlacement(
      event.shiftKey
        ? {
            ...widget.placement,
            columnSpan: Math.max(1, widget.placement.columnSpan + deltaColumns),
            rowSpan: Math.max(1, widget.placement.rowSpan + deltaRows)
          }
        : {
            ...widget.placement,
            column: widget.placement.column + deltaColumns,
            row: widget.placement.row + deltaRows
          }
    );

    if (isAreaFree(buildOccupancy(layout.widgets, widget.widgetId), candidate)) {
      commitPlacement(widget.widgetId, candidate);
    }
  }
</script>

<div
  class="relative aspect-video w-full overflow-hidden bg-surface-1 select-none"
  bind:clientWidth={surfaceWidth}
  bind:clientHeight={surfaceHeight}
>
  <WidgetSurface {layout} {pin} {joinUrl} {mediaBaseUrl} />

  <div
    class="absolute inset-0 grid"
    style={`
      grid-template-columns: repeat(${WIDGET_GRID_COLUMNS}, minmax(0, 1fr));
      grid-template-rows: repeat(${WIDGET_GRID_ROWS}, minmax(0, 1fr));
      gap: 1.2%;
      padding: 1.6%;
    `}
  >
    {#if drag}
      {#each Array(WIDGET_GRID_COLUMNS * WIDGET_GRID_ROWS) as _, index}
        <div
          class="pointer-events-none bg-white/5"
          style={`grid-column: ${(index % WIDGET_GRID_COLUMNS) + 1}; grid-row: ${Math.floor(index / WIDGET_GRID_COLUMNS) + 1};`}
        ></div>
      {/each}
    {/if}

    {#each visibleWidgets as widget (widget.widgetId)}
      {@const placement = placementFor(widget)}
      <div
        class="group relative cursor-grab touch-none transition-colors hover:bg-white/5"
        class:cursor-grabbing={drag?.widgetId === widget.widgetId}
        class:outline={selectedWidgetId === widget.widgetId || drag?.widgetId === widget.widgetId}
        class:outline-2={selectedWidgetId === widget.widgetId || drag?.widgetId === widget.widgetId}
        class:outline-accent={isDraftValid}
        class:outline-danger={!isDraftValid && drag?.widgetId === widget.widgetId}
        style={`
          grid-column: ${placement.column} / span ${placement.columnSpan};
          grid-row: ${placement.row} / span ${placement.rowSpan};
        `}
        role="button"
        tabindex="0"
        aria-label={`Move ${widget.kind} widget`}
        onpointerdown={(event) => beginDrag(event, widget, "move")}
        onpointermove={updateDrag}
        onpointerup={endDrag}
        onpointercancel={endDrag}
        onkeydown={(event) => nudge(event, widget)}
        onfocus={() => onSelect(widget.widgetId)}
      >
        <span
          class="absolute top-1 left-1 bg-surface-0/80 px-2 py-0.5 text-xs text-ink-1 opacity-0 transition-opacity group-hover:opacity-100"
        >
          {widget.kind}
        </span>

        <button
          class="absolute right-0 bottom-0 h-5 w-5 cursor-nwse-resize bg-accent opacity-0 transition-opacity group-hover:opacity-100"
          class:opacity-100={selectedWidgetId === widget.widgetId}
          aria-label={`Resize ${widget.kind} widget`}
          onpointerdown={(event) => beginDrag(event, widget, "resize")}
          onpointermove={updateDrag}
          onpointerup={endDrag}
          onpointercancel={endDrag}
        ></button>
      </div>
    {/each}
  </div>
</div>
