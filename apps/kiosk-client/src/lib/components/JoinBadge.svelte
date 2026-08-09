<script lang="ts">
  import { formatPin } from "@pistation/shared-types";

  let {
    pin,
    joinUrl
  }: {
    pin: string | null;
    joinUrl: string;
  } = $props();
</script>

{#if pin}
  <div class="absolute top-6 right-6 flex flex-col items-end gap-1 bg-surface-0/80 px-5 py-3">
    {#if joinUrl}
      <p class="text-xs tracking-wide text-ink-2 uppercase">Join at {joinUrl}</p>
    {:else}
      <p class="text-xs tracking-wide text-ink-2 uppercase">Join code</p>
    {/if}
    <p class="font-mono text-3xl leading-none font-bold tracking-[0.08em] text-ink-0">
      {formatPin(pin)}
    </p>
  </div>
{/if}
