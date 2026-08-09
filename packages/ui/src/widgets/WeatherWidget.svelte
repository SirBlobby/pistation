<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { WeatherSettings } from "@pistation/shared-types";
  import { onMount } from "svelte";

  let { settings }: { settings: WeatherSettings } = $props();

  interface Reading {
    temperature: number;
    weatherCode: number;
  }

  let reading = $state<Reading | null>(null);
  let hasFailed = $state(false);

  const WEATHER_ICONS: Record<number, string> = {
    0: "ph:sun-bold",
    1: "ph:sun-dim-bold",
    2: "ph:cloud-sun-bold",
    3: "ph:cloud-bold",
    45: "ph:cloud-fog-bold",
    48: "ph:cloud-fog-bold",
    51: "ph:cloud-rain-bold",
    61: "ph:cloud-rain-bold",
    63: "ph:cloud-rain-bold",
    65: "ph:cloud-rain-bold",
    71: "ph:cloud-snow-bold",
    73: "ph:cloud-snow-bold",
    75: "ph:cloud-snow-bold",
    80: "ph:cloud-rain-bold",
    95: "ph:cloud-lightning-bold",
    96: "ph:cloud-lightning-bold"
  };

  const icon = $derived(reading ? (WEATHER_ICONS[reading.weatherCode] ?? "ph:cloud-bold") : "ph:cloud-bold");
  const unitLabel = $derived(settings.units === "imperial" ? "F" : "C");

  async function load() {
    const temperatureUnit = settings.units === "imperial" ? "fahrenheit" : "celsius";
    const url =
      `https://api.open-meteo.com/v1/forecast?latitude=${settings.latitude}` +
      `&longitude=${settings.longitude}&current=temperature_2m,weather_code` +
      `&temperature_unit=${temperatureUnit}`;

    try {
      const response = await fetch(url);
      if (!response.ok) throw new Error("weather request failed");

      const payload = (await response.json()) as {
        current?: { temperature_2m?: number; weather_code?: number };
      };

      if (payload.current?.temperature_2m === undefined) throw new Error("no reading");

      reading = {
        temperature: Math.round(payload.current.temperature_2m),
        weatherCode: payload.current.weather_code ?? 0
      };
      hasFailed = false;
    } catch {
      hasFailed = true;
    }
  }

  onMount(() => {
    void load();
    const interval = setInterval(() => void load(), 15 * 60 * 1000);
    return () => clearInterval(interval);
  });
</script>

<div
  class="flex h-full min-w-0 gap-[4cqmin] overflow-hidden"
  style="justify-content: var(--widget-align, flex-start); align-items: var(--widget-valign, center);"
>
  <span class="shrink-0 opacity-80" style="font-size: clamp(1rem, 26cqmin, 7rem);">
    <Icon {icon} width="1em" height="1em" />
  </span>

  <div class="min-w-0">
    {#if reading}
      <p class="truncate leading-none font-semibold" style="font-size: clamp(1rem, 26cqmin, 7rem);">
        {reading.temperature}&deg;{unitLabel}
      </p>
    {:else}
      <p class="truncate opacity-60" style="font-size: clamp(0.7rem, 10cqmin, 2rem);">
        {hasFailed ? "Weather unavailable" : "Loading"}
      </p>
    {/if}
    <p class="truncate opacity-60" style="font-size: clamp(0.6rem, 8cqmin, 1.75rem);">
      {settings.locationLabel}
    </p>
  </div>
</div>
