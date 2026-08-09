<script lang="ts">
  import Icon from "@iconify/svelte";
  import type {
    AgendaSettings,
    ClockSettings,
    CustomSettings,
    CustomWidgetDefinition,
    ImageSettings,
    PinSettings,
    TextSettings,
    WeatherSettings,
    Widget,
    WidgetStyle
  } from "@pistation/shared-types";
  import {
    DEFAULT_WIDGET_OPACITY,
    DEFAULT_WIDGET_STYLE,
    WIDGET_GRID_COLUMNS,
    WIDGET_GRID_ROWS
  } from "@pistation/shared-types";

  import TimeZoneField from "./TimeZoneField.svelte";

  let {
    widget,
    definitions,
    layoutOpacity = DEFAULT_WIDGET_OPACITY,
    onChange,
    onRemove
  }: {
    widget: Widget;
    definitions: CustomWidgetDefinition[];
    layoutOpacity?: number;
    onChange: (widget: Widget) => void;
    onRemove: () => void;
  } = $props();

  const ALIGN_OPTIONS = [
    {
      value: "start" as const,
      label: "Start",
      horizontalIcon: "ph:align-left-simple-bold",
      verticalIcon: "ph:align-top-simple-bold"
    },
    {
      value: "center" as const,
      label: "Centre",
      horizontalIcon: "ph:align-center-horizontal-simple-bold",
      verticalIcon: "ph:align-center-vertical-simple-bold"
    },
    {
      value: "end" as const,
      label: "End",
      horizontalIcon: "ph:align-right-simple-bold",
      verticalIcon: "ph:align-bottom-simple-bold"
    }
  ];

  const style = $derived({ ...DEFAULT_WIDGET_STYLE, ...(widget.style ?? {}) });

  function patchSettings(patch: Record<string, unknown>) {
    onChange({ ...widget, settings: { ...widget.settings, ...patch } as Widget["settings"] });
  }

  function patchStyle(patch: Partial<WidgetStyle>) {
    onChange({ ...widget, style: { ...style, ...patch } });
  }

  function patchPlacement(patch: Record<string, number>) {
    onChange({ ...widget, placement: { ...widget.placement, ...patch } });
  }

  const clock = $derived(widget.settings as ClockSettings);
  const weather = $derived(widget.settings as WeatherSettings);
  const pin = $derived(widget.settings as PinSettings);
  const text = $derived(widget.settings as TextSettings);
  const image = $derived(widget.settings as ImageSettings);
  const agenda = $derived(widget.settings as AgendaSettings);
  const custom = $derived(widget.settings as CustomSettings);
</script>

<div class="flex flex-col gap-5 bg-surface-1 p-5">
  <div class="flex items-center gap-2">
    <h3 class="flex-1 text-sm font-semibold tracking-wide uppercase">{widget.kind}</h3>
    <label class="flex items-center gap-2 text-sm text-ink-1">
      <input
        type="checkbox"
        checked={widget.enabled}
        onchange={(event) =>
          onChange({ ...widget, enabled: (event.target as HTMLInputElement).checked })}
      />
      Shown
    </label>
    <button
      onclick={onRemove}
      aria-label="Remove widget"
      class="flex h-8 w-8 items-center justify-center bg-surface-2 text-danger hover:bg-surface-3"
    >
      <Icon icon="ph:trash-bold" width="16" />
    </button>
  </div>

  <div class="grid grid-cols-4 gap-3">
    {#each [
      { key: "column", label: "Col", max: WIDGET_GRID_COLUMNS },
      { key: "row", label: "Row", max: WIDGET_GRID_ROWS },
      { key: "columnSpan", label: "Width", max: WIDGET_GRID_COLUMNS },
      { key: "rowSpan", label: "Height", max: WIDGET_GRID_ROWS }
    ] as field}
      <label class="block">
        <span class="mb-1 block text-xs text-ink-2">{field.label}</span>
        <input
          type="number"
          min="1"
          max={field.max}
          value={widget.placement[field.key as keyof typeof widget.placement]}
          oninput={(event) =>
            patchPlacement({
              [field.key]: Number((event.target as HTMLInputElement).value)
            })}
          class="w-full bg-surface-2 px-3 py-2 text-sm"
        />
      </label>
    {/each}
  </div>

  <div class="flex flex-col gap-4 bg-surface-2 p-4">
    <p class="text-xs tracking-wide text-ink-2 uppercase">Style</p>

    <label class="block">
      <span class="mb-1 block text-xs text-ink-2">Padding {style.padding}</span>
      <input
        type="range"
        min="0"
        max="20"
        step="0.5"
        value={style.padding}
        oninput={(event) =>
          patchStyle({ padding: Number((event.target as HTMLInputElement).value) })}
        class="w-full"
      />
    </label>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <span class="mb-1 block text-xs text-ink-2">Horizontal</span>
        <div class="flex gap-px">
          {#each ALIGN_OPTIONS as option}
            <button
              onclick={() => patchStyle({ align: option.value })}
              aria-label={option.label}
              title={option.label}
              class="flex h-9 flex-1 items-center justify-center transition-colors"
              class:bg-accent={style.align === option.value}
              class:text-white={style.align === option.value}
              class:bg-surface-1={style.align !== option.value}
              class:text-ink-1={style.align !== option.value}
            >
              <Icon icon={option.horizontalIcon} width="16" />
            </button>
          {/each}
        </div>
      </div>

      <div>
        <span class="mb-1 block text-xs text-ink-2">Vertical</span>
        <div class="flex gap-px">
          {#each ALIGN_OPTIONS as option}
            <button
              onclick={() => patchStyle({ verticalAlign: option.value })}
              aria-label={option.label}
              title={option.label}
              class="flex h-9 flex-1 items-center justify-center transition-colors"
              class:bg-accent={style.verticalAlign === option.value}
              class:text-white={style.verticalAlign === option.value}
              class:bg-surface-1={style.verticalAlign !== option.value}
              class:text-ink-1={style.verticalAlign !== option.value}
            >
              <Icon icon={option.verticalIcon} width="16" />
            </button>
          {/each}
        </div>
      </div>
    </div>

    <div class="flex items-center justify-between gap-3">
      <span class="text-xs text-ink-2">Panel colour</span>
      <div class="flex items-center gap-2">
        {#if style.backgroundColor}
          <button
            onclick={() => patchStyle({ backgroundColor: "" })}
            class="bg-surface-1 px-3 py-1.5 text-xs text-ink-1 hover:bg-surface-3"
          >
            Reset
          </button>
        {/if}
        <input
          type="color"
          value={style.backgroundColor || "#ffffff"}
          oninput={(event) =>
            patchStyle({ backgroundColor: (event.target as HTMLInputElement).value })}
          class="h-9 w-14 bg-surface-1"
        />
      </div>
    </div>

    <label class="block">
      <span class="mb-1 block text-xs text-ink-2">
        Panel opacity
        {style.opacity === null ? "follows the layout" : `${Math.round(style.opacity * 100)} percent`}
      </span>
      <div class="flex items-center gap-2">
        <input
          type="range"
          min="0"
          max="100"
          value={(style.opacity ?? layoutOpacity) * 100}
          oninput={(event) =>
            patchStyle({ opacity: Number((event.target as HTMLInputElement).value) / 100 })}
          class="w-full"
        />
        {#if style.opacity !== null}
          <button
            onclick={() => patchStyle({ opacity: null })}
            class="shrink-0 bg-surface-1 px-3 py-1.5 text-xs text-ink-1 hover:bg-surface-3"
          >
            Reset
          </button>
        {/if}
      </div>
    </label>
  </div>

  {#if widget.kind === "clock"}
    <TimeZoneField
      value={clock.timeZone}
      onChange={(timeZone) => patchSettings({ timeZone })}
    />
    <div class="flex gap-4 text-sm text-ink-1">
      <label class="flex items-center gap-2">
        <input
          type="checkbox"
          checked={clock.showSeconds}
          onchange={(event) =>
            patchSettings({ showSeconds: (event.target as HTMLInputElement).checked })}
        />
        Seconds
      </label>
      <label class="flex items-center gap-2">
        <input
          type="checkbox"
          checked={clock.showDate}
          onchange={(event) =>
            patchSettings({ showDate: (event.target as HTMLInputElement).checked })}
        />
        Date
      </label>
      <label class="flex items-center gap-2">
        <input
          type="checkbox"
          checked={clock.hour12}
          onchange={(event) =>
            patchSettings({ hour12: (event.target as HTMLInputElement).checked })}
        />
        12 hour
      </label>
    </div>
  {:else if widget.kind === "weather"}
    <div class="grid grid-cols-2 gap-3">
      <label class="block">
        <span class="mb-1 block text-xs text-ink-2">Latitude</span>
        <input
          type="number"
          step="0.0001"
          value={weather.latitude}
          oninput={(event) =>
            patchSettings({ latitude: Number((event.target as HTMLInputElement).value) })}
          class="w-full bg-surface-2 px-3 py-2 text-sm"
        />
      </label>
      <label class="block">
        <span class="mb-1 block text-xs text-ink-2">Longitude</span>
        <input
          type="number"
          step="0.0001"
          value={weather.longitude}
          oninput={(event) =>
            patchSettings({ longitude: Number((event.target as HTMLInputElement).value) })}
          class="w-full bg-surface-2 px-3 py-2 text-sm"
        />
      </label>
    </div>
    <label class="block">
      <span class="mb-1 block text-xs text-ink-2">Location label</span>
      <input
        value={weather.locationLabel}
        oninput={(event) =>
          patchSettings({ locationLabel: (event.target as HTMLInputElement).value })}
        class="w-full bg-surface-2 px-3 py-2 text-sm"
      />
    </label>
    <label class="block">
      <span class="mb-1 block text-xs text-ink-2">Units</span>
      <select
        value={weather.units}
        onchange={(event) => patchSettings({ units: (event.target as HTMLSelectElement).value })}
        class="w-full bg-surface-2 px-3 py-2 text-sm"
      >
        <option value="metric">Celsius</option>
        <option value="imperial">Fahrenheit</option>
      </select>
    </label>
  {:else if widget.kind === "pin"}
    <label class="block">
      <span class="mb-1 block text-xs text-ink-2">Label</span>
      <input
        value={pin.label}
        oninput={(event) => patchSettings({ label: (event.target as HTMLInputElement).value })}
        class="w-full bg-surface-2 px-3 py-2 text-sm"
      />
    </label>
    <label class="flex items-center gap-2 text-sm text-ink-1">
      <input
        type="checkbox"
        checked={pin.showJoinUrl}
        onchange={(event) =>
          patchSettings({ showJoinUrl: (event.target as HTMLInputElement).checked })}
      />
      Show join address
    </label>
  {:else if widget.kind === "text"}
    <label class="block">
      <span class="mb-1 block text-xs text-ink-2">Heading</span>
      <input
        value={text.heading}
        oninput={(event) => patchSettings({ heading: (event.target as HTMLInputElement).value })}
        class="w-full bg-surface-2 px-3 py-2 text-sm"
      />
    </label>
    <label class="block">
      <span class="mb-1 block text-xs text-ink-2">Body</span>
      <textarea
        value={text.body}
        rows="4"
        oninput={(event) => patchSettings({ body: (event.target as HTMLTextAreaElement).value })}
        class="w-full bg-surface-2 px-3 py-2 text-sm"
      ></textarea>
    </label>
  {:else if widget.kind === "image"}
    <label class="block">
      <span class="mb-1 block text-xs text-ink-2">Image URL</span>
      <input
        value={image.imageUrl}
        oninput={(event) => patchSettings({ imageUrl: (event.target as HTMLInputElement).value })}
        class="w-full bg-surface-2 px-3 py-2 text-sm"
      />
    </label>
    <label class="block">
      <span class="mb-1 block text-xs text-ink-2">Fit</span>
      <select
        value={image.fit}
        onchange={(event) => patchSettings({ fit: (event.target as HTMLSelectElement).value })}
        class="w-full bg-surface-2 px-3 py-2 text-sm"
      >
        <option value="cover">Cover</option>
        <option value="contain">Contain</option>
      </select>
    </label>
  {:else if widget.kind === "agenda"}
    <label class="block">
      <span class="mb-1 block text-xs text-ink-2">Heading</span>
      <input
        value={agenda.heading}
        oninput={(event) => patchSettings({ heading: (event.target as HTMLInputElement).value })}
        class="w-full bg-surface-2 px-3 py-2 text-sm"
      />
    </label>
    <label class="block">
      <span class="mb-1 block text-xs text-ink-2">Items, one per line</span>
      <textarea
        value={agenda.items.join("\n")}
        rows="5"
        oninput={(event) =>
          patchSettings({
            items: (event.target as HTMLTextAreaElement).value
              .split("\n")
              .map((line) => line.trim())
              .filter(Boolean)
          })}
        class="w-full bg-surface-2 px-3 py-2 text-sm"
      ></textarea>
    </label>
  {:else if widget.kind === "custom"}
    <label class="block">
      <span class="mb-1 block text-xs text-ink-2">Definition</span>
      <select
        value={custom.definitionId}
        onchange={(event) =>
          patchSettings({ definitionId: (event.target as HTMLSelectElement).value })}
        class="w-full bg-surface-2 px-3 py-2 text-sm"
      >
        <option value="">Choose a custom widget</option>
        {#each definitions as definition}
          <option value={definition.definitionId}>{definition.name}</option>
        {/each}
      </select>
    </label>
  {/if}
</div>
