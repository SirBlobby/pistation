<script lang="ts">
  import type { KioskLayout, NightModeSettings } from "@pistation/shared-types";
  import { isNightModeActive } from "@pistation/shared-types";

  import TimeZoneField from "./TimeZoneField.svelte";

  let {
    layout,
    onChange
  }: {
    layout: KioskLayout;
    onChange: (layout: KioskLayout) => void;
  } = $props();

  const nightMode = $derived(layout.nightMode);
  const isActiveNow = $derived(isNightModeActive(nightMode));

  function patch(update: Partial<NightModeSettings>) {
    onChange({ ...layout, nightMode: { ...layout.nightMode, ...update } });
  }
</script>

<div class="flex flex-col gap-4 bg-surface-1 p-5">
  <div class="flex items-center gap-3">
    <h3 class="flex-1 text-sm font-semibold tracking-wide text-ink-2 uppercase">Night mode</h3>
    {#if nightMode.enabled && isActiveNow}
      <span class="bg-accent px-2 py-1 text-xs font-medium text-white">Active now</span>
    {/if}
  </div>

  <p class="text-sm text-ink-2">
    Between these times the screen shows only the clock and the PIN. Widgets, wallpaper and
    all other graphics are hidden. A live presentation always takes priority.
  </p>

  <label class="flex items-center gap-3 text-sm text-ink-1">
    <input
      type="checkbox"
      checked={nightMode.enabled}
      onchange={(event) => patch({ enabled: (event.target as HTMLInputElement).checked })}
    />
    Enable night mode
  </label>

  {#if nightMode.enabled}
    <div class="grid grid-cols-2 gap-3">
      <label class="block">
        <span class="mb-1 block text-xs text-ink-2">Starts</span>
        <input
          type="time"
          value={nightMode.startTime}
          oninput={(event) => patch({ startTime: (event.target as HTMLInputElement).value })}
          class="w-full bg-surface-2 px-3 py-2 text-sm"
        />
      </label>

      <label class="block">
        <span class="mb-1 block text-xs text-ink-2">Ends</span>
        <input
          type="time"
          value={nightMode.endTime}
          oninput={(event) => patch({ endTime: (event.target as HTMLInputElement).value })}
          class="w-full bg-surface-2 px-3 py-2 text-sm"
        />
      </label>
    </div>

    <TimeZoneField
      value={nightMode.timeZone}
      emptyLabel="Kiosk clock"
      onChange={(timeZone) => patch({ timeZone })}
    />

    <label class="flex items-center gap-3 text-sm text-ink-1">
      <input
        type="checkbox"
        checked={nightMode.showPin}
        onchange={(event) => patch({ showPin: (event.target as HTMLInputElement).checked })}
      />
      Show the PIN at night
    </label>

    <label class="block">
      <span class="mb-1 block text-xs text-ink-2">
        Brightness {Math.round(nightMode.brightness * 100)} percent
      </span>
      <input
        type="range"
        min="5"
        max="100"
        value={nightMode.brightness * 100}
        oninput={(event) =>
          patch({ brightness: Number((event.target as HTMLInputElement).value) / 100 })}
        class="w-full"
      />
    </label>
  {/if}
</div>
