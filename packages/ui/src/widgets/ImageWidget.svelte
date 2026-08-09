<script lang="ts">
  import type { ImageSettings } from "@pistation/shared-types";

  let { settings }: { settings: ImageSettings } = $props();
</script>

{#if settings.imageUrl}
  <img
    src={settings.imageUrl}
    alt=""
    class="h-full w-full"
    class:object-cover={settings.fit === "cover"}
    class:object-contain={settings.fit === "contain"}
  />
{:else}
  <div class="flex h-full items-center justify-center text-sm opacity-40">No image set</div>
{/if}
