<script lang="ts">
  import type { KioskLayout, Widget, WidgetStyle } from "@pistation/shared-types";
  import {
    DEFAULT_WIDGET_OPACITY,
    DEFAULT_WIDGET_STYLE,
    MIN_WALLPAPER_ROTATION_SECONDS,
    resolveMediaUrl,
    toCssColor,
    toFlexAlign,
    toTextAlign,
    WIDGET_GRID_COLUMNS,
    WIDGET_GRID_ROWS
  } from "@pistation/shared-types";

  import WidgetRenderer from "./WidgetRenderer.svelte";

  let {
    layout,
    pin = null,
    joinUrl = "",
    mediaBaseUrl = "",
    selectedWidgetId = null,
    onSelect
  }: {
    layout: KioskLayout;
    pin?: string | null;
    joinUrl?: string;
    mediaBaseUrl?: string;
    selectedWidgetId?: string | null;
    onSelect?: (widgetId: string) => void;
  } = $props();

  const visibleWidgets = $derived(layout.widgets.filter((widget) => widget.enabled));
  const dim = $derived(Math.min(1, Math.max(0, layout.background.dim)));

  const wallpapers = $derived(
    (layout.background.images ?? []).map((image) => resolveMediaUrl(mediaBaseUrl, image))
  );
  const hasWallpaper = $derived(wallpapers.length > 0);

  let activeWallpaper = $state(0);

  // Every wallpaper stays mounted so the browser has it decoded before its turn, and the
  // change is a crossfade rather than a blink.
  $effect(() => {
    const total = wallpapers.length;
    if (total <= 1) {
      activeWallpaper = 0;
      return;
    }

    const seconds = Math.max(
      MIN_WALLPAPER_ROTATION_SECONDS,
      layout.background.rotationSeconds ?? 60
    );

    const interval = setInterval(() => {
      activeWallpaper = (activeWallpaper + 1) % total;
    }, seconds * 1000);

    return () => clearInterval(interval);
  });
  const widgetOpacity = $derived(
    Math.min(1, Math.max(0, layout.widgetOpacity ?? DEFAULT_WIDGET_OPACITY))
  );

  function styleOf(widget: Widget): WidgetStyle {
    return { ...DEFAULT_WIDGET_STYLE, ...(widget.style ?? {}) };
  }

  function cellBackground(style: WidgetStyle): string {
    const opacity = Math.min(1, Math.max(0, style.opacity ?? widgetOpacity));

    if (style.backgroundColor) return toCssColor(style.backgroundColor, opacity);
    if (hasWallpaper) return `rgb(0 0 0 / ${opacity})`;
    return `rgb(255 255 255 / ${(opacity * 0.16).toFixed(3)})`;
  }
</script>

<div
  class="relative h-full w-full overflow-hidden"
  style={`background-color: ${layout.backgroundColor}; color: ${layout.foregroundColor};`}
>
  {#if hasWallpaper}
    {#each wallpapers as wallpaper, index (wallpaper)}
      <img
        src={wallpaper}
        alt=""
        class="absolute inset-0 h-full w-full transition-opacity duration-1000"
        class:object-cover={layout.background.fit === "cover"}
        class:object-contain={layout.background.fit === "contain"}
        style={`opacity: ${index === activeWallpaper ? 1 : 0};`}
      />
    {/each}
    <div class="absolute inset-0 bg-black" style={`opacity: ${dim};`}></div>
  {/if}

  <div
    class="relative grid h-full w-full"
    style={`
      grid-template-columns: repeat(${WIDGET_GRID_COLUMNS}, minmax(0, 1fr));
      grid-template-rows: repeat(${WIDGET_GRID_ROWS}, minmax(0, 1fr));
      gap: 1.2%;
      padding: 1.6%;
    `}
  >
  {#each visibleWidgets as widget (widget.widgetId)}
    {@const style = styleOf(widget)}
    {@const interaction = onSelect
      ? {
          role: "button",
          tabindex: 0,
          onclick: () => onSelect(widget.widgetId),
          onkeydown: (event: KeyboardEvent) => {
            if (event.key === "Enter") onSelect(widget.widgetId);
          }
        }
      : {}}
    <div
      {...interaction}
      class="relative min-h-0 min-w-0 overflow-hidden"
      class:cursor-pointer={Boolean(onSelect)}
      class:outline={selectedWidgetId === widget.widgetId}
      class:outline-2={selectedWidgetId === widget.widgetId}
      style={`
        grid-column: ${widget.placement.column} / span ${widget.placement.columnSpan};
        grid-row: ${widget.placement.row} / span ${widget.placement.rowSpan};
        background-color: ${cellBackground(style)};
        container-type: size;
      `}
    >
      <div
        class="h-full w-full overflow-hidden"
        style={`
          padding: ${style.padding}cqmin;
          text-align: ${toTextAlign(style.align)};
          --widget-align: ${toFlexAlign(style.align)};
          --widget-valign: ${toFlexAlign(style.verticalAlign)};
        `}
      >
        <WidgetRenderer {widget} {pin} {joinUrl} definitions={layout.customDefinitions} />
      </div>
    </div>
    {/each}
  </div>
</div>
