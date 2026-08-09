<script lang="ts">
  import type { ClockSettings } from "@pistation/shared-types";
  import { safeTimeZone } from "@pistation/shared-types";
  import { onMount } from "svelte";

  let { settings }: { settings: ClockSettings } = $props();

  let now = $state(new Date());

  onMount(() => {
    const interval = setInterval(() => (now = new Date()), settings.showSeconds ? 1000 : 15000);
    return () => clearInterval(interval);
  });

  const timeZone = $derived(safeTimeZone(settings.timeZone));

  const timeLabel = $derived(
    now.toLocaleTimeString([], {
      timeZone,
      hour: "2-digit",
      minute: "2-digit",
      second: settings.showSeconds ? "2-digit" : undefined,
      hour12: settings.hour12
    })
  );

  const dateLabel = $derived(
    now.toLocaleDateString([], {
      timeZone,
      weekday: "long",
      month: "long",
      day: "numeric"
    })
  );
</script>

<div
  class="flex h-full min-w-0 flex-col overflow-hidden"
  style="justify-content: var(--widget-valign, center);"
>
  <p
    class="truncate font-mono leading-none font-semibold tracking-tight"
    style="font-size: clamp(0.9rem, 30cqmin, 11rem);"
  >
    {timeLabel}
  </p>
  {#if settings.showDate}
    <p class="mt-[2cqmin] truncate opacity-60" style="font-size: clamp(0.6rem, 8cqmin, 2rem);">
      {dateLabel}
    </p>
  {/if}
</div>
