<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { BrandLink, OrganizationBranding } from "@pistation/shared-types";
  import {
    createId,
    DEFAULT_THEME,
    resolveMediaUrl,
    withBrandingDefaults
  } from "@pistation/shared-types";
  import { onMount } from "svelte";

  import { ApiError, apiBaseUrl, getBranding, saveBranding, uploadLogo } from "$lib/api";
  import { applyBranding } from "$lib/branding.svelte";
  import { loadAdmin } from "$lib/session";

  const admin = loadAdmin();

  const LANDING_OPTIONS = [
    {
      value: "full" as const,
      label: "Full homepage",
      description: "Marketing sections, features and footer"
    },
    {
      value: "join" as const,
      label: "Join only",
      description: "Just the logo and the code entry"
    }
  ];

  const THEME_FIELDS = [
    { key: "surface0" as const, label: "Page background" },
    { key: "surface1" as const, label: "Panels" },
    { key: "surface2" as const, label: "Inputs" },
    { key: "surface3" as const, label: "Borders and hovers" },
    { key: "ink0" as const, label: "Primary text" },
    { key: "ink1" as const, label: "Secondary text" },
    { key: "ink2" as const, label: "Muted text" }
  ];

  let branding = $state<OrganizationBranding | null>(null);
  let statusMessage = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let isSaving = $state(false);
  let isUploading = $state(false);
  let logoInput = $state<HTMLInputElement | null>(null);

  const logoPreview = $derived(
    branding?.logoUrl ? resolveMediaUrl(apiBaseUrl, branding.logoUrl) : ""
  );

  onMount(() => {
    void load();
  });

  async function load() {
    try {
      branding = withBrandingDefaults(await getBranding());
    } catch {
      branding = withBrandingDefaults(null);
      errorMessage = "Could not load the current branding, showing defaults.";
    }
  }

  function patch(update: Partial<OrganizationBranding>) {
    if (!branding) return;
    branding = { ...branding, ...update };
  }

  async function handleLogo(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (!file || !admin) return;

    isUploading = true;
    errorMessage = null;

    try {
      const result = await uploadLogo(admin.accessToken, file);
      patch({ logoUrl: result.imageUrl });
      statusMessage = "Logo uploaded. Save to publish it.";
    } catch (error) {
      errorMessage = error instanceof ApiError ? error.message : "Could not upload the logo.";
    } finally {
      isUploading = false;
    }
  }

  function addLink() {
    if (!branding) return;
    const link: BrandLink = { linkId: createId("link"), label: "", url: "" };
    patch({ links: [...branding.links, link] });
  }

  function updateLink(linkId: string, update: Partial<BrandLink>) {
    if (!branding) return;
    patch({
      links: branding.links.map((link) => (link.linkId === linkId ? { ...link, ...update } : link))
    });
  }

  function removeLink(linkId: string) {
    if (!branding) return;
    patch({ links: branding.links.filter((link) => link.linkId !== linkId) });
  }

  async function save() {
    if (!admin || !branding) return;

    isSaving = true;
    statusMessage = null;
    errorMessage = null;

    try {
      branding = withBrandingDefaults(await saveBranding(admin.accessToken, branding));
      // Push it into the shared store so the theme changes under you straight away.
      applyBranding(branding);
      statusMessage = "Branding saved and applied.";
    } catch (error) {
      errorMessage = error instanceof ApiError ? error.message : "Could not save the branding.";
    } finally {
      isSaving = false;
    }
  }
</script>

<svelte:head>
  <title>Organisation · Admin · PiStation</title>
  <meta name="robots" content="noindex, nofollow" />
</svelte:head>

<main class="mx-auto w-full max-w-4xl px-6 py-12">
  <header class="mb-8 flex flex-wrap items-center gap-4">
    <a
      href="/admin"
      aria-label="Back to kiosks"
      class="flex h-10 w-10 items-center justify-center bg-surface-2 text-ink-1 hover:bg-surface-3"
    >
      <Icon icon="ph:arrow-left-bold" width="18" />
    </a>

    <div class="flex-1">
      <h1 class="text-xl font-semibold tracking-tight">Organisation</h1>
      <p class="text-sm text-ink-2">Branding shown on the public homepage</p>
    </div>

    <button
      onclick={save}
      disabled={isSaving || !branding}
      class="flex items-center gap-2 bg-accent px-5 py-3 font-semibold text-white transition-colors hover:bg-accent-strong disabled:bg-surface-3 disabled:text-ink-2"
    >
      <Icon icon="ph:floppy-disk-bold" width="18" />
      Save
    </button>
  </header>

  {#if statusMessage}
    <p class="mb-6 flex items-center gap-2 bg-success/15 px-4 py-3 text-sm text-success">
      <Icon icon="ph:check-circle-bold" width="18" class="shrink-0" />
      {statusMessage}
    </p>
  {/if}

  {#if errorMessage}
    <p class="mb-6 flex items-center gap-2 bg-danger/15 px-4 py-3 text-sm text-danger">
      <Icon icon="ph:warning-circle-bold" width="18" class="shrink-0" />
      {errorMessage}
    </p>
  {/if}

  {#if branding}
    {@const current = branding}
    <div class="flex flex-col gap-6">
      <section class="flex flex-col gap-4 bg-surface-1 p-6">
        <h2 class="text-sm font-semibold tracking-wide text-ink-2 uppercase">Identity</h2>

        <label class="block">
          <span class="mb-1 block text-xs text-ink-2">Organisation name</span>
          <input
            value={current.name}
            oninput={(event) => patch({ name: (event.target as HTMLInputElement).value })}
            class="w-full bg-surface-2 px-4 py-3 text-ink-0"
          />
        </label>

        <div class="flex flex-wrap items-end gap-4">
          <div class="flex items-center gap-3">
            {#if logoPreview}
              <img src={logoPreview} alt="Logo" class="h-14 w-14 object-contain" />
            {:else}
              <div class="flex h-14 w-14 items-center justify-center bg-surface-2 text-ink-2">
                <Icon icon="ph:image-bold" width="22" />
              </div>
            {/if}

            <input
              bind:this={logoInput}
              type="file"
              accept="image/png,image/jpeg,image/webp,image/gif"
              onchange={handleLogo}
              class="hidden"
            />

            <button
              onclick={() => logoInput?.click()}
              disabled={isUploading}
              class="flex items-center gap-2 bg-surface-2 px-4 py-2 text-sm text-ink-1 hover:bg-surface-3 disabled:text-ink-2"
            >
              {#if isUploading}
                <Icon icon="ph:circle-notch-bold" width="16" class="animate-spin" />
                Uploading
              {:else}
                <Icon icon="ph:upload-simple-bold" width="16" />
                Upload logo
              {/if}
            </button>

            {#if current.logoUrl}
              <button
                onclick={() => patch({ logoUrl: "" })}
                class="px-3 py-2 text-sm text-danger hover:bg-surface-2"
              >
                Reset
              </button>
            {/if}
          </div>

          <label class="ml-auto flex items-center gap-3 text-sm text-ink-1">
            Accent colour
            <input
              type="color"
              value={current.accentColor}
              oninput={(event) =>
                patch({ accentColor: (event.target as HTMLInputElement).value })}
              class="h-10 w-16 bg-surface-2"
            />
          </label>
        </div>
      </section>

      <section class="flex flex-col gap-4 bg-surface-1 p-6">
        <h2 class="text-sm font-semibold tracking-wide text-ink-2 uppercase">Homepage</h2>

        <div class="flex flex-col gap-2">
          <span class="text-xs text-ink-2">What visitors see at the root address</span>
          <div class="flex gap-px">
            {#each LANDING_OPTIONS as option}
              <button
                onclick={() => patch({ landingMode: option.value })}
                class="flex-1 px-4 py-3 text-left transition-colors"
                class:bg-accent={current.landingMode === option.value}
                class:text-white={current.landingMode === option.value}
                class:bg-surface-2={current.landingMode !== option.value}
                class:text-ink-1={current.landingMode !== option.value}
              >
                <span class="block text-sm font-medium">{option.label}</span>
                <span class="block text-xs opacity-70">{option.description}</span>
              </button>
            {/each}
          </div>
        </div>

        <label class="block">
          <span class="mb-1 block text-xs text-ink-2">Headline</span>
          <input
            value={current.headline}
            oninput={(event) => patch({ headline: (event.target as HTMLInputElement).value })}
            class="w-full bg-surface-2 px-4 py-3 text-ink-0"
          />
        </label>

        <label class="block">
          <span class="mb-1 block text-xs text-ink-2">Description</span>
          <textarea
            value={current.description}
            rows="3"
            oninput={(event) =>
              patch({ description: (event.target as HTMLTextAreaElement).value })}
            class="w-full bg-surface-2 px-4 py-3 text-ink-0"
          ></textarea>
        </label>

        <label class="block">
          <span class="mb-1 block text-xs text-ink-2">Label above the join code</span>
          <input
            value={current.joinLabel}
            oninput={(event) => patch({ joinLabel: (event.target as HTMLInputElement).value })}
            class="w-full bg-surface-2 px-4 py-3 text-ink-0"
          />
        </label>
      </section>

      <section class="flex flex-col gap-4 bg-surface-1 p-6">
        <div class="flex items-center gap-3">
          <h2 class="flex-1 text-sm font-semibold tracking-wide text-ink-2 uppercase">Theme</h2>
          <button
            onclick={() => patch({ theme: { ...DEFAULT_THEME } })}
            class="bg-surface-2 px-3 py-2 text-sm text-ink-1 hover:bg-surface-3"
          >
            Reset
          </button>
        </div>

        <p class="text-sm text-ink-2">
          These colours apply across the whole website, including the join screen and the admin
          panel.
        </p>

        <div class="grid gap-3 sm:grid-cols-2">
          {#each THEME_FIELDS as field}
            <label class="flex items-center justify-between gap-3 bg-surface-2 px-4 py-3">
              <span class="text-sm text-ink-1">{field.label}</span>
              <input
                type="color"
                value={current.theme[field.key]}
                oninput={(event) =>
                  patch({
                    theme: {
                      ...current.theme,
                      [field.key]: (event.target as HTMLInputElement).value
                    }
                  })}
                class="h-9 w-16 bg-surface-1"
              />
            </label>
          {/each}
        </div>
      </section>

      <section class="flex flex-col gap-4 bg-surface-1 p-6">
        <div class="flex items-center gap-3">
          <h2 class="flex-1 text-sm font-semibold tracking-wide text-ink-2 uppercase">Footer</h2>
          <button
            onclick={addLink}
            class="flex items-center gap-2 bg-surface-2 px-3 py-2 text-sm text-ink-1 hover:bg-surface-3"
          >
            <Icon icon="ph:plus-bold" width="16" />
            Add link
          </button>
        </div>

        <label class="block">
          <span class="mb-1 block text-xs text-ink-2">Footer note</span>
          <input
            value={current.footerNote}
            placeholder="Internal use only, contact IT for help"
            oninput={(event) => patch({ footerNote: (event.target as HTMLInputElement).value })}
            class="w-full bg-surface-2 px-4 py-3 text-ink-0 placeholder:text-ink-2"
          />
        </label>

        {#each current.links as link (link.linkId)}
          <div class="flex flex-wrap items-center gap-2">
            <input
              value={link.label}
              placeholder="Label"
              oninput={(event) =>
                updateLink(link.linkId, { label: (event.target as HTMLInputElement).value })}
              class="min-w-32 flex-1 bg-surface-2 px-3 py-2 text-sm"
            />
            <input
              value={link.url}
              placeholder="https://"
              oninput={(event) =>
                updateLink(link.linkId, { url: (event.target as HTMLInputElement).value })}
              class="min-w-48 flex-2 bg-surface-2 px-3 py-2 text-sm"
            />
            <button
              onclick={() => removeLink(link.linkId)}
              aria-label="Remove link"
              class="flex h-9 w-9 items-center justify-center bg-surface-2 text-danger hover:bg-surface-3"
            >
              <Icon icon="ph:trash-bold" width="16" />
            </button>
          </div>
        {/each}

        <label class="flex items-center gap-3 text-sm text-ink-1">
          <input
            type="checkbox"
            checked={current.showSourceLink}
            onchange={(event) =>
              patch({ showSourceLink: (event.target as HTMLInputElement).checked })}
          />
          Show the source link in the footer
        </label>
      </section>
    </div>
  {/if}
</main>
