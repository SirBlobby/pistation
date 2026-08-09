<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { KioskMetrics } from "@pistation/shared-types";
  import {
    formatBytes,
    formatUptime,
    signalAdvice,
    signalLabel,
    signalPercent
  } from "@pistation/shared-types";

  let {
    metrics,
    metricsAt,
    compact = false
  }: {
    metrics: KioskMetrics | null;
    metricsAt: number | null;
    compact?: boolean;
  } = $props();

  const SIGNAL_COLORS = {
    excellent: "text-success",
    good: "text-success",
    weak: "text-accent",
    poor: "text-danger"
  } as const;

  const SIGNAL_BARS = {
    excellent: "bg-success",
    good: "bg-success",
    weak: "bg-accent",
    poor: "bg-danger"
  } as const;

  const WIFI_ICONS = {
    excellent: "ph:wifi-high-bold",
    good: "ph:wifi-high-bold",
    weak: "ph:wifi-medium-bold",
    poor: "ph:wifi-low-bold"
  } as const;

  const STRENGTH_WORDS = {
    excellent: "Excellent",
    good: "Good",
    weak: "Weak",
    poor: "Poor"
  } as const;

  const memoryPercent = $derived(
    metrics && metrics.memoryTotalBytes > 0
      ? (metrics.memoryUsedBytes / metrics.memoryTotalBytes) * 100
      : null
  );

  const isStale = $derived(metricsAt !== null && Date.now() - metricsAt > 120_000);

  function barColor(percent: number): string {
    if (percent >= 90) return "bg-danger";
    if (percent >= 70) return "bg-accent";
    return "bg-success";
  }
</script>

{#if !metrics}
  <p class="text-sm text-ink-2">No stats reported yet.</p>
{:else if compact}
  <div class="flex flex-wrap items-center gap-4 text-sm text-ink-2">
    {#if metrics.cpuPercent !== null}
      <span class="flex items-center gap-1.5">
        <Icon icon="ph:cpu-bold" width="14" />
        {metrics.cpuPercent.toFixed(0)}%
      </span>
    {/if}

    {#if memoryPercent !== null}
      <span class="flex items-center gap-1.5">
        <Icon icon="ph:memory-bold" width="14" />
        {memoryPercent.toFixed(0)}%
      </span>
    {/if}

    {#if metrics.wifi}
      {@const strength = signalLabel(metrics.wifi.signalDbm)}
      <span
        class={`flex items-center gap-1.5 ${SIGNAL_COLORS[strength]}`}
        title={`${metrics.wifi.signalDbm.toFixed(0)} dBm on ${metrics.wifi.interface}`}
      >
        <Icon icon={WIFI_ICONS[strength]} width="14" />
        {STRENGTH_WORDS[strength]}
      </span>
    {/if}

    {#if metrics.temperatureCelsius !== null}
      <span class="flex items-center gap-1.5">
        <Icon icon="ph:thermometer-simple-bold" width="14" />
        {metrics.temperatureCelsius.toFixed(0)}&deg;C
      </span>
    {/if}
  </div>
{:else}
  <div class="flex flex-col gap-4">
    {#if isStale}
      <p class="flex items-center gap-2 bg-surface-2 px-3 py-2 text-xs text-ink-2">
        <Icon icon="ph:clock-countdown-bold" width="14" />
        These figures are more than two minutes old.
      </p>
    {/if}

    {#if metrics.cpuPercent !== null}
      <div>
        <div class="mb-1 flex items-baseline justify-between text-sm">
          <span class="flex items-center gap-2 text-ink-1">
            <Icon icon="ph:cpu-bold" width="16" />
            CPU
          </span>
          <span class="text-ink-2">{metrics.cpuPercent.toFixed(0)}%</span>
        </div>
        <div class="h-1.5 w-full bg-surface-2">
          <div
            class={`h-full ${barColor(metrics.cpuPercent)}`}
            style={`width: ${Math.min(100, metrics.cpuPercent)}%`}
          ></div>
        </div>
      </div>
    {/if}

    {#if memoryPercent !== null}
      <div>
        <div class="mb-1 flex items-baseline justify-between text-sm">
          <span class="flex items-center gap-2 text-ink-1">
            <Icon icon="ph:memory-bold" width="16" />
            Memory
          </span>
          <span class="text-ink-2">
            {formatBytes(metrics.memoryUsedBytes)} of {formatBytes(metrics.memoryTotalBytes)}
          </span>
        </div>
        <div class="h-1.5 w-full bg-surface-2">
          <div
            class={`h-full ${barColor(memoryPercent)}`}
            style={`width: ${Math.min(100, memoryPercent)}%`}
          ></div>
        </div>
      </div>
    {/if}

    {#if metrics.wifi}
      {@const strength = signalLabel(metrics.wifi.signalDbm)}
      {@const percent = signalPercent(metrics.wifi.signalDbm)}
      <div>
        <div class="mb-1 flex items-baseline justify-between text-sm">
          <span class="flex items-center gap-2 text-ink-1">
            <Icon icon={WIFI_ICONS[strength]} width="16" class={SIGNAL_COLORS[strength]} />
            Wi-Fi
          </span>
          <span class={SIGNAL_COLORS[strength]}>
            {STRENGTH_WORDS[strength]}
            <span class="text-ink-2">{percent}%</span>
          </span>
        </div>
        <div class="h-1.5 w-full bg-surface-2">
          <div class={`h-full ${SIGNAL_BARS[strength]}`} style={`width: ${percent}%`}></div>
        </div>
        <p class="mt-1 text-xs text-ink-2">
          {signalAdvice(metrics.wifi.signalDbm)} · {metrics.wifi.signalDbm.toFixed(0)} dBm on
          {metrics.wifi.interface}
        </p>
      </div>
    {:else}
      <div class="flex items-baseline justify-between text-sm">
        <span class="flex items-center gap-2 text-ink-1">
          <Icon icon="ph:network-bold" width="16" />
          Network
        </span>
        <span class="text-ink-2">Wired or no wireless adapter</span>
      </div>
    {/if}

    {#if metrics.temperatureCelsius !== null}
      <div class="flex items-baseline justify-between text-sm">
        <span class="flex items-center gap-2 text-ink-1">
          <Icon icon="ph:thermometer-simple-bold" width="16" />
          Temperature
        </span>
        <span class={metrics.temperatureCelsius >= 75 ? "text-danger" : "text-ink-2"}>
          {metrics.temperatureCelsius.toFixed(1)}&deg;C
        </span>
      </div>
    {/if}

    <div class="flex items-baseline justify-between text-sm">
      <span class="flex items-center gap-2 text-ink-1">
        <Icon icon="ph:timer-bold" width="16" />
        Uptime
      </span>
      <span class="text-ink-2">{formatUptime(metrics.uptimeSeconds)}</span>
    </div>
  </div>
{/if}
