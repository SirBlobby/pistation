<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { Widget } from "@pistation/shared-types";

  let {
    widgets,
    selectedWidgetId,
    onSelect,
    onChange
  }: {
    widgets: Widget[];
    selectedWidgetId: string | null;
    onSelect: (widgetId: string) => void;
    onChange: (widgets: Widget[]) => void;
  } = $props();

  function move(index: number, offset: number) {
    const target = index + offset;
    if (target < 0 || target >= widgets.length) return;

    const reordered = [...widgets];
    const [moved] = reordered.splice(index, 1);
    reordered.splice(target, 0, moved);
    onChange(reordered);
  }

  function toggle(widgetId: string) {
    onChange(
      widgets.map((widget) =>
        widget.widgetId === widgetId ? { ...widget, enabled: !widget.enabled } : widget
      )
    );
  }

  function remove(widgetId: string) {
    onChange(widgets.filter((widget) => widget.widgetId !== widgetId));
  }
</script>

<div class="flex flex-col gap-3 bg-surface-1 p-5">
  <h3 class="text-sm font-semibold tracking-wide text-ink-2 uppercase">Widgets</h3>

  {#if widgets.length === 0}
    <p class="py-4 text-center text-sm text-ink-2">No widgets yet.</p>
  {/if}

  <div class="flex flex-col gap-px">
    {#each widgets as widget, index (widget.widgetId)}
      <div
        class="flex items-center gap-2 px-3 py-2 transition-colors"
        class:bg-surface-2={selectedWidgetId !== widget.widgetId}
        class:bg-accent={selectedWidgetId === widget.widgetId}
      >
        <button
          onclick={() => onSelect(widget.widgetId)}
          class="flex-1 text-left text-sm capitalize"
          class:text-white={selectedWidgetId === widget.widgetId}
          class:text-ink-1={selectedWidgetId !== widget.widgetId}
          class:opacity-40={!widget.enabled}
        >
          {widget.kind}
          <span class="ml-2 text-xs opacity-60">
            {widget.placement.columnSpan} by {widget.placement.rowSpan}
          </span>
        </button>

        <button
          onclick={() => toggle(widget.widgetId)}
          aria-label={widget.enabled ? "Hide widget" : "Show widget"}
          class="flex h-7 w-7 items-center justify-center text-ink-1 hover:bg-surface-3"
        >
          <Icon icon={widget.enabled ? "ph:eye-bold" : "ph:eye-slash-bold"} width="14" />
        </button>

        <button
          onclick={() => move(index, -1)}
          aria-label="Move earlier"
          class="flex h-7 w-7 items-center justify-center text-ink-1 hover:bg-surface-3"
        >
          <Icon icon="ph:arrow-up-bold" width="14" />
        </button>

        <button
          onclick={() => move(index, 1)}
          aria-label="Move later"
          class="flex h-7 w-7 items-center justify-center text-ink-1 hover:bg-surface-3"
        >
          <Icon icon="ph:arrow-down-bold" width="14" />
        </button>

        <button
          onclick={() => remove(widget.widgetId)}
          aria-label="Delete widget"
          class="flex h-7 w-7 items-center justify-center text-danger hover:bg-surface-3"
        >
          <Icon icon="ph:trash-bold" width="14" />
        </button>
      </div>
    {/each}
  </div>
</div>
