<script lang="ts">
  import type { KioskLayout } from "@pistation/shared-types";
  import { formatPin, safeTimeZone } from "@pistation/shared-types";
  import { onMount } from "svelte";

  let {
    layout,
    pin = null
  }: {
    layout: KioskLayout;
    pin?: string | null;
  } = $props();

  let now = $state(new Date());

  onMount(() => {
    const interval = setInterval(() => (now = new Date()), 20000);
    return () => clearInterval(interval);
  });

  const clockSettings = $derived(
    layout.widgets.find((widget) => widget.kind === "clock")?.settings as
      | { timeZone?: string; hour12?: boolean }
      | undefined
  );

  const timeLabel = $derived(
    now.toLocaleTimeString([], {
      timeZone: safeTimeZone(clockSettings?.timeZone ?? ""),
      hour: "2-digit",
      minute: "2-digit",
      hour12: clockSettings?.hour12 ?? true
    })
  );

  const brightness = $derived(Math.min(1, Math.max(0.05, layout.nightMode.brightness)));
</script>

<div
  class="flex h-full w-full flex-col items-center justify-center overflow-hidden bg-black"
  style={`container-type: size; opacity: ${brightness}; color: ${layout.foregroundColor};`}
>
  <p
    class="truncate font-mono leading-none font-semibold tracking-tight"
    style="font-size: clamp(2rem, 22cqmin, 20rem);"
  >
    {timeLabel}
  </p>

  {#if layout.nightMode.showPin && pin}
    <p
      class="mt-[4cqmin] truncate font-mono leading-none tracking-[0.12em] opacity-70"
      style="font-size: clamp(1rem, 9cqmin, 8rem);"
    >
      {formatPin(pin)}
    </p>
  {/if}
</div>
