<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { KioskLayout } from "@pistation/shared-types";
  import { MIN_WALLPAPER_ROTATION_SECONDS, resolveMediaUrl } from "@pistation/shared-types";

  import { apiBaseUrl } from "$lib/api";

  let {
    layout,
    onChange,
    onUpload,
    isUploading = false,
    uploadError = null
  }: {
    layout: KioskLayout;
    onChange: (layout: KioskLayout) => void;
    onUpload: (files: File[]) => void;
    isUploading?: boolean;
    uploadError?: string | null;
  } = $props();

  let fileInput = $state<HTMLInputElement | null>(null);

  const images = $derived(layout.background.images ?? []);

  function patchBackground(patch: Partial<KioskLayout["background"]>) {
    onChange({ ...layout, background: { ...layout.background, ...patch } });
  }

  function removeImage(image: string) {
    patchBackground({ images: images.filter((candidate) => candidate !== image) });
  }

  function handleFile(event: Event) {
    const input = event.target as HTMLInputElement;
    const files = [...(input.files ?? [])];
    if (files.length > 0) onUpload(files);
    input.value = "";
  }
</script>

<div class="flex flex-col gap-4 bg-surface-1 p-5">
  <h3 class="text-sm font-semibold tracking-wide text-ink-2 uppercase">Appearance</h3>

  <label class="flex items-center justify-between gap-3 text-sm text-ink-1">
    Background colour
    <input
      type="color"
      value={layout.backgroundColor}
      oninput={(event) =>
        onChange({ ...layout, backgroundColor: (event.target as HTMLInputElement).value })}
      class="h-9 w-16 bg-surface-2"
    />
  </label>

  <label class="flex items-center justify-between gap-3 text-sm text-ink-1">
    Text colour
    <input
      type="color"
      value={layout.foregroundColor}
      oninput={(event) =>
        onChange({ ...layout, foregroundColor: (event.target as HTMLInputElement).value })}
      class="h-9 w-16 bg-surface-2"
    />
  </label>

  <label class="block">
    <span class="mb-1 block text-xs text-ink-2">
      Widget panel opacity {Math.round((layout.widgetOpacity ?? 0.5) * 100)} percent
    </span>
    <input
      type="range"
      min="0"
      max="100"
      value={(layout.widgetOpacity ?? 0.5) * 100}
      oninput={(event) =>
        onChange({
          ...layout,
          widgetOpacity: Number((event.target as HTMLInputElement).value) / 100
        })}
      class="w-full"
    />
  </label>

  <div class="flex flex-col gap-3 bg-surface-2 p-4">
    <div class="flex items-center gap-2">
      <p class="flex-1 text-xs tracking-wide text-ink-2 uppercase">Wallpapers</p>
      {#if images.length > 0}
        <span class="text-xs text-ink-2">{images.length}</span>
      {/if}
    </div>

    {#if images.length === 0}
      <p class="bg-surface-1 px-4 py-6 text-center text-sm text-ink-2">No wallpaper set</p>
    {:else}
      <div class="grid grid-cols-3 gap-2">
        {#each images as image, index (image)}
          <div class="group relative">
            <img
              src={resolveMediaUrl(apiBaseUrl, image)}
              alt={`Wallpaper ${index + 1}`}
              class="h-16 w-full object-cover"
            />
            <button
              onclick={() => removeImage(image)}
              aria-label={`Remove wallpaper ${index + 1}`}
              class="absolute top-0 right-0 flex h-6 w-6 items-center justify-center bg-surface-0/80 text-danger opacity-0 transition-opacity group-hover:opacity-100"
            >
              <Icon icon="ph:x-bold" width="12" />
            </button>
          </div>
        {/each}
      </div>
    {/if}

    {#if uploadError}
      <p class="bg-danger/15 px-3 py-2 text-sm text-danger">{uploadError}</p>
    {/if}

    <input
      bind:this={fileInput}
      type="file"
      accept="image/png,image/jpeg,image/webp,image/gif"
      multiple
      onchange={handleFile}
      class="hidden"
    />

    <button
      onclick={() => fileInput?.click()}
      disabled={isUploading}
      class="flex items-center justify-center gap-2 bg-surface-1 px-4 py-2 text-sm text-ink-1 transition-colors hover:bg-surface-3 disabled:text-ink-2"
    >
      {#if isUploading}
        <Icon icon="ph:circle-notch-bold" width="16" class="animate-spin" />
        Uploading
      {:else}
        <Icon icon="ph:upload-simple-bold" width="16" />
        Add images
      {/if}
    </button>

    <p class="text-xs text-ink-2">PNG, JPEG, WEBP or GIF, up to 8 MB each.</p>

    {#if images.length > 1}
      <label class="block">
        <span class="mb-1 block text-xs text-ink-2">
          Change every {layout.background.rotationSeconds ?? 60} seconds
        </span>
        <input
          type="range"
          min={MIN_WALLPAPER_ROTATION_SECONDS}
          max="600"
          step="5"
          value={layout.background.rotationSeconds ?? 60}
          oninput={(event) =>
            patchBackground({
              rotationSeconds: Number((event.target as HTMLInputElement).value)
            })}
          class="w-full"
        />
      </label>
    {/if}

    {#if images.length > 0}
      <label class="block">
        <span class="mb-1 block text-xs text-ink-2">Fit</span>
        <select
          value={layout.background.fit}
          onchange={(event) =>
            patchBackground({
              fit: (event.target as HTMLSelectElement).value as "cover" | "contain"
            })}
          class="w-full bg-surface-1 px-3 py-2 text-sm"
        >
          <option value="cover">Cover</option>
          <option value="contain">Contain</option>
        </select>
      </label>

      <label class="block">
        <span class="mb-1 block text-xs text-ink-2">
          Dim {Math.round(layout.background.dim * 100)} percent
        </span>
        <input
          type="range"
          min="0"
          max="100"
          value={layout.background.dim * 100}
          oninput={(event) =>
            patchBackground({ dim: Number((event.target as HTMLInputElement).value) / 100 })}
          class="w-full"
        />
      </label>
    {/if}
  </div>
</div>
