<script lang="ts">
  import { themeStyle } from "@pistation/shared-types";
  import { onMount } from "svelte";
  import { page } from "$app/stores";

  import { branding, ensureBranding } from "$lib/branding.svelte";
  import { SITE_DESCRIPTION, SITE_NAME, THEME_COLOR } from "$lib/meta";
  import "../app.css";

  let { children } = $props();

  onMount(() => {
    void ensureBranding();
  });

  const canonicalUrl = $derived(`${$page.url.origin}${$page.url.pathname}`);
  const imageUrl = $derived(`${$page.url.origin}/og-image.svg`);
</script>

<svelte:head>
  <meta name="theme-color" content={THEME_COLOR} />
  <meta name="color-scheme" content="dark" />
  <link rel="canonical" href={canonicalUrl} />

  <meta property="og:site_name" content={SITE_NAME} />
  <meta property="og:type" content="website" />
  <meta property="og:url" content={canonicalUrl} />
  <meta property="og:image" content={imageUrl} />
  <meta property="og:image:width" content="1200" />
  <meta property="og:image:height" content="630" />
  <meta property="og:image:alt" content={SITE_NAME} />

  <meta name="twitter:card" content="summary_large_image" />
  <meta name="twitter:image" content={imageUrl} />

  <meta name="description" content={SITE_DESCRIPTION} />
  <meta property="og:description" content={SITE_DESCRIPTION} />
  <meta name="twitter:description" content={SITE_DESCRIPTION} />
</svelte:head>

<div class="min-h-full" style={themeStyle(branding.value)}>
  {@render children()}
</div>
