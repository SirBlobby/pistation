<script lang="ts">
  import Icon from "@iconify/svelte";
  import {
    AnnotationOverlay,
    Logo,
    NightSurface,
    WhiteboardCanvas,
    WidgetSurface
  } from "@pistation/ui";
  import { onDestroy, onMount } from "svelte";

  import { invoke } from "@tauri-apps/api/core";

  import JoinBadge from "$lib/components/JoinBadge.svelte";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import VideoCanvas from "$lib/components/VideoCanvas.svelte";
  import { KioskController } from "$lib/kiosk.svelte";

  let controller = $state<KioskController | null>(null);
  let isKioskMode = $state(true);

  const participantLabels = $derived(
    new Map((controller?.participants ?? []).map((p) => [p.identity, p.displayName]))
  );

  onMount(() => {
    void invoke<boolean>("is_kiosk_mode")
      .then((value) => (isKioskMode = value))
      .catch(() => (isKioskMode = true));

    const instance = new KioskController();
    controller = instance;
    void instance.start();
  });

  onDestroy(() => {
    void controller?.stop();
  });
</script>

{#if controller}
  {@const kiosk = controller}
  {@const isPresenting = kiosk.mode === "presentation" && kiosk.isScreenActive}
  <div class="flex h-screen w-screen flex-col overflow-hidden bg-surface-0">
    {#if !isKioskMode}
      <TitleBar status={kiosk.status} />
    {/if}

    <div class="relative min-h-0 flex-1 overflow-hidden">
    <VideoCanvas visible={isPresenting} />

    {#if kiosk.provisioningError}
      <div class="flex h-full flex-col items-center justify-center gap-6 px-16 text-center">
        <Logo size={96} />
        <div>
          <h1 class="text-3xl font-semibold">This kiosk is not set up yet</h1>
          <p class="mt-3 max-w-2xl text-lg text-ink-2">{kiosk.provisioningError}</p>
        </div>
        <p class="text-sm text-ink-2">
          Add the server URL and enrollment token to /boot/firmware/pistation.json and restart.
        </p>
      </div>
    {:else if kiosk.mode === "whiteboard"}
      <WhiteboardCanvas
        elements={kiosk.whiteboardElements}
        files={kiosk.resolvedWhiteboardFiles}
        readOnly={true}
      />
    {:else if isPresenting}
      <AnnotationOverlay annotations={kiosk.annotations} labels={participantLabels} />
    {:else if kiosk.layout && kiosk.isNight}
      <NightSurface layout={kiosk.layout} pin={kiosk.pin} />
    {:else if kiosk.layout}
      <WidgetSurface
        layout={kiosk.layout}
        pin={kiosk.pin}
        joinUrl={kiosk.joinUrl}
        mediaBaseUrl={kiosk.mediaBaseUrl}
      />
    {:else}
      <div class="flex h-full items-center justify-center">
        <Icon icon="ph:circle-notch-bold" width="48" class="animate-spin text-ink-2" />
      </div>
    {/if}

    {#if (isPresenting || kiosk.mode === "whiteboard") && !kiosk.provisioningError}
      <JoinBadge pin={kiosk.pin} joinUrl={kiosk.joinUrl} />
    {/if}

    {#if kiosk.participants.length > 0 && !kiosk.provisioningError}
      <div
        class="absolute bottom-6 left-6 flex items-center gap-2 bg-surface-1/90 px-4 py-2.5 text-ink-1"
      >
        <Icon icon="ph:users-bold" width="18" />
        <span class="text-lg font-semibold text-ink-0">{kiosk.participants.length}</span>
        <span class="text-sm">
          {kiosk.participants.length === 1 ? "person connected" : "people connected"}
        </span>
      </div>
    {/if}

    {#if !kiosk.provisioningError && kiosk.status !== "connected" && kiosk.status !== "idle"}
      <div class="absolute right-6 bottom-6 flex max-w-lg items-start gap-2 bg-surface-1 px-4 py-2">
        <Icon
          icon={kiosk.status === "connecting" ? "ph:circle-notch-bold" : "ph:wifi-slash-bold"}
          width="18"
          class={kiosk.status === "connecting" ? "mt-0.5 animate-spin text-ink-2" : "mt-0.5 text-danger"}
        />
        <div class="min-w-0">
          <span class="text-sm text-ink-1">
            {kiosk.status === "connecting" ? "Connecting" : "Reconnecting"}
          </span>
          {#if kiosk.connectionError}
            <p class="text-xs break-words text-ink-2">{kiosk.connectionError}</p>
          {/if}
        </div>
      </div>
    {/if}
    </div>
  </div>
{/if}
