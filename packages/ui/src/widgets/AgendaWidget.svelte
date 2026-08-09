<script lang="ts">
  import type { AgendaSettings } from "@pistation/shared-types";

  let { settings }: { settings: AgendaSettings } = $props();
</script>

<div
  class="flex h-full min-w-0 flex-col gap-[3cqmin] overflow-hidden"
  style="justify-content: var(--widget-valign, center);"
>
  {#if settings.heading}
    <h3
      class="truncate font-semibold tracking-wide uppercase opacity-60"
      style="font-size: clamp(0.65rem, 9cqmin, 1.75rem);"
    >
      {settings.heading}
    </h3>
  {/if}

  <ul class="flex min-h-0 flex-col gap-[2cqmin] overflow-hidden">
    {#each settings.items as item}
      <li
        class="flex items-start gap-[2.5cqmin]"
        style="font-size: clamp(0.65rem, 8cqmin, 1.5rem); justify-content: var(--widget-align, flex-start);"
      >
        <span class="mt-[0.55em] h-[0.3em] w-[0.3em] shrink-0 bg-current opacity-50"></span>
        <span class="min-w-0">{item}</span>
      </li>
    {/each}
  </ul>
</div>
