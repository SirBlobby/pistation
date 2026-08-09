<script lang="ts">
  import type { PinSettings } from "@pistation/shared-types";
  import { formatPin } from "@pistation/shared-types";

  let {
    settings,
    pin,
    joinUrl
  }: {
    settings: PinSettings;
    pin: string | null;
    joinUrl: string;
  } = $props();
</script>

<div
  class="flex h-full min-w-0 flex-col overflow-hidden"
  style="justify-content: var(--widget-valign, center);"
>
  <p
    class="w-full truncate tracking-wide uppercase opacity-60"
    style="font-size: clamp(0.6rem, 6cqmin, 1.75rem);"
  >
    {settings.label}
  </p>

  {#if settings.showJoinUrl}
    <p
      class="mt-[1cqmin] w-full truncate font-medium opacity-90"
      style="font-size: clamp(0.7rem, 8cqmin, 2.5rem);"
    >
      {joinUrl}
    </p>
  {/if}

  {#if pin}
    <p
      class="mt-[3cqmin] w-full truncate font-mono leading-none font-bold tracking-[0.08em]"
      style="font-size: clamp(1.5rem, 34cqmin, 15rem);"
    >
      {formatPin(pin)}
    </p>
  {:else}
    <p class="mt-[3cqmin] w-full opacity-40" style="font-size: clamp(1rem, 18cqmin, 5rem);">
      ------
    </p>
  {/if}
</div>
