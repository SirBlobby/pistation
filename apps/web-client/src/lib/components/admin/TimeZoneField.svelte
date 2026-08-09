<script lang="ts">
  import { isValidTimeZone, listTimeZones, localTimeZone } from "@pistation/shared-types";

  let {
    value,
    label = "Time zone",
    emptyLabel = "Device clock",
    onChange
  }: {
    value: string;
    label?: string;
    emptyLabel?: string;
    onChange: (value: string) => void;
  } = $props();

  const zones = listTimeZones();

  const groups = $derived.by(() => {
    const byRegion = new Map<string, string[]>();

    for (const zone of zones) {
      const region = zone.includes("/") ? zone.split("/")[0] : "Other";
      const existing = byRegion.get(region);
      if (existing) existing.push(zone);
      else byRegion.set(region, [zone]);
    }

    return [...byRegion.entries()].sort((a, b) => a[0].localeCompare(b[0]));
  });

  const isKnown = $derived(!value || zones.includes(value));

  const preview = $derived.by(() => {
    if (value && !isValidTimeZone(value)) return null;
    try {
      return new Date().toLocaleTimeString([], {
        timeZone: value || undefined,
        hour: "2-digit",
        minute: "2-digit"
      });
    } catch {
      return null;
    }
  });

  function labelFor(zone: string): string {
    return zone.includes("/") ? zone.split("/").slice(1).join("/").replace(/_/g, " ") : zone;
  }
</script>

<label class="block">
  <span class="mb-1 block text-xs text-ink-2">{label}</span>

  <select
    {value}
    onchange={(event) => onChange((event.target as HTMLSelectElement).value)}
    class="w-full bg-surface-2 px-3 py-2 text-sm text-ink-0"
  >
    <option value="">{emptyLabel} ({localTimeZone()})</option>

    {#if !isKnown}
      <option value={value}>{value}</option>
    {/if}

    {#each groups as [region, regionZones]}
      <optgroup label={region}>
        {#each regionZones as zone}
          <option value={zone}>{labelFor(zone)}</option>
        {/each}
      </optgroup>
    {/each}
  </select>

  {#if preview}
    <span class="mt-1 block text-xs text-ink-2">Currently {preview} there</span>
  {/if}
</label>
