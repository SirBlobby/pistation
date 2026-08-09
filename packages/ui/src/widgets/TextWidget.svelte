<script lang="ts">
  import type { TextSettings } from "@pistation/shared-types";

  let { settings }: { settings: TextSettings } = $props();
</script>

<div
  class="flex h-full min-w-0 flex-col overflow-hidden"
  style="justify-content: var(--widget-valign, center);"
>
  {#if settings.heading}
    <h3 class="font-semibold" style="font-size: clamp(0.8rem, 14cqmin, 3rem);">
      {settings.heading}
    </h3>
  {/if}
  {#if settings.body}
    <p
      class="mt-[2cqmin] whitespace-pre-line opacity-70"
      style="font-size: clamp(0.65rem, 9cqmin, 1.75rem);"
    >
      {settings.body}
    </p>
  {/if}
</div>
