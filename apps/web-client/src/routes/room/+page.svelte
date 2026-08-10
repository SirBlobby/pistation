<script lang="ts">
  import Icon from "@iconify/svelte";
  import { onDestroy, onMount } from "svelte";
  import { goto } from "$app/navigation";

  import { activeAuthors } from "@pistation/client-core/annotations";
  import { AnnotationOverlay, Logo, TrackVideo, WhiteboardCanvas } from "@pistation/ui";

  import AnnotationToolbar from "$lib/components/AnnotationToolbar.svelte";
  import ConnectionBadge from "$lib/components/ConnectionBadge.svelte";
  import ParticipantMenu from "$lib/components/ParticipantMenu.svelte";
  import { RoomController } from "$lib/room.svelte";
  import { clearSession, loadSession } from "$lib/session";

  let controller = $state<RoomController | null>(null);
  let isAnnotating = $state(false);

  const participantLabels = $derived(
    new Map((controller?.participants ?? []).map((p) => [p.participantId, p.displayName]))
  );

  const screenBlocked = $derived(
    Boolean(controller && !controller.isSharing && !controller.canStartScreenShare)
  );
  const cameraBlocked = $derived(
    Boolean(controller && !controller.isCameraOn && !controller.canStartCamera)
  );

  const cameraTiles = $derived.by(() => {
    if (!controller) return [];

    const remote = controller.cameraFeeds.map((feed) => ({
      key: feed.participantId,
      label: feed.displayName,
      track: feed.track
    }));

    if (!controller.localCameraTrack) return remote;

    return [
      { key: "self", label: "You", track: controller.localCameraTrack },
      ...remote
    ];
  });

  const drawingNow = $derived.by(() => {
    if (!controller) return [];
    return activeAuthors(controller.annotations)
      .filter((authorId) => authorId !== controller?.participantId)
      .map((authorId) => participantLabels.get(authorId) ?? "Someone");
  });

  onMount(() => {
    const session = loadSession();
    if (!session) {
      void goto("/");
      return;
    }

    const instance = new RoomController(session);
    controller = instance;
    void instance.connect();
  });

  onDestroy(() => {
    void controller?.disconnect();
  });

  async function leave() {
    await controller?.disconnect();
    clearSession();
    await goto("/");
  }

  function toggleShare() {
    if (!controller) return;
    if (controller.isSharing) {
      void controller.stopScreenShare();
    } else {
      void controller.startScreenShare();
    }
  }

  function toggleCamera() {
    if (!controller) return;
    if (controller.isCameraOn) {
      void controller.stopCamera();
    } else {
      void controller.startCamera();
    }
  }

  function toggleWhiteboard() {
    if (!controller) return;
    controller.setMode(controller.mode === "whiteboard" ? "idle" : "whiteboard");
  }
</script>

<svelte:head>
  <title>{controller ? `${controller.kioskName} · PiStation` : "Room · PiStation"}</title>
  <meta name="robots" content="noindex, nofollow" />
</svelte:head>

{#if controller}
  {@const current = controller}
  <div class="flex h-screen flex-col">
    <header class="flex shrink-0 flex-wrap items-center gap-3 bg-surface-1 px-4 py-3">
      <div class="flex items-center gap-3">
        <Logo size={32} />
        <div>
          <p class="text-sm font-semibold">{current.kioskName}</p>
          <p class="text-xs text-ink-2">
            {current.role === "presenter" ? "Presenting" : "Viewing"}
          </p>
        </div>
      </div>

      <ConnectionBadge status={current.status} />

      <ParticipantMenu participants={current.participants} />

      <div class="ml-auto flex flex-wrap items-center gap-2">
        <button
          onclick={toggleShare}
          disabled={screenBlocked}
          title={screenBlocked ? (current.blockedReason ?? "Turn your camera off first") : undefined}
          class="flex items-center gap-2 px-4 py-2 text-sm font-medium transition-colors disabled:bg-surface-2 disabled:text-ink-2"
          class:bg-danger={current.isSharing}
          class:text-white={current.isSharing}
          class:bg-accent={!current.isSharing && !screenBlocked}
          class:hover:bg-accent-strong={!current.isSharing && !screenBlocked}
        >
          <Icon icon={current.isSharing ? "ph:stop-circle-bold" : "ph:broadcast-bold"} width="18" />
          {current.isSharing ? "Stop sharing" : "Share screen"}
        </button>

        <button
          onclick={toggleCamera}
          disabled={cameraBlocked}
          title={cameraBlocked
            ? (current.blockedReason ?? "Stop sharing your screen first")
            : undefined}
          class="flex items-center gap-2 px-4 py-2 text-sm font-medium transition-colors disabled:bg-surface-2 disabled:text-ink-2"
          class:bg-danger={current.isCameraOn}
          class:text-white={current.isCameraOn}
          class:bg-surface-2={!current.isCameraOn && cameraBlocked}
          class:bg-accent={!current.isCameraOn && !cameraBlocked}
          class:hover:bg-accent-strong={!current.isCameraOn && !cameraBlocked}
        >
          <Icon
            icon={current.isCameraOn ? "ph:video-camera-slash-bold" : "ph:video-camera-bold"}
            width="18"
          />
          {current.isCameraOn ? "Stop camera" : "Camera"}
        </button>

        {#if current.isCameraOn}
          <button
            onclick={() => current.flipCamera()}
            aria-label="Switch camera"
            title="Switch camera"
            class="flex h-9 w-9 items-center justify-center bg-surface-2 text-ink-1 transition-colors hover:bg-surface-3"
          >
            <Icon icon="ph:arrows-clockwise-bold" width="16" />
          </button>
        {/if}

        <button
          onclick={toggleWhiteboard}
          disabled={current.mode === "whiteboard" && !current.canStopWhiteboard}
          title={current.mode === "whiteboard" && !current.canStopWhiteboard
            ? `${current.whiteboardOwnerName ?? "Someone else"} opened this whiteboard and can close it`
            : undefined}
          class="flex items-center gap-2 px-4 py-2 text-sm font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-60"
          class:bg-accent={current.mode === "whiteboard"}
          class:text-white={current.mode === "whiteboard"}
          class:bg-surface-2={current.mode !== "whiteboard"}
          class:text-ink-1={current.mode !== "whiteboard"}
          class:hover:bg-surface-3={current.mode !== "whiteboard"}
        >
          <Icon icon="ph:scribble-loop-bold" width="18" />
          Whiteboard
        </button>

        <button
          onclick={() => (isAnnotating = !isAnnotating)}
          disabled={current.mode === "whiteboard"}
          class="flex items-center gap-2 px-4 py-2 text-sm font-medium transition-colors disabled:bg-surface-2 disabled:text-ink-2"
          class:bg-accent={isAnnotating && current.mode !== "whiteboard"}
          class:text-white={isAnnotating && current.mode !== "whiteboard"}
          class:bg-surface-2={!isAnnotating}
          class:text-ink-1={!isAnnotating}
        >
          <Icon icon="ph:pencil-simple-bold" width="18" />
          Annotate
        </button>

        <button
          onclick={leave}
          class="flex items-center gap-2 bg-surface-2 px-4 py-2 text-sm font-medium text-danger transition-colors hover:bg-surface-3"
        >
          <Icon icon="ph:sign-out-bold" width="18" />
          Leave
        </button>
      </div>
    </header>

    {#if current.errorMessage}
      <p class="flex items-center gap-2 bg-danger/15 px-4 py-2 text-sm text-danger">
        <Icon icon="ph:warning-circle-bold" width="16" />
        {current.errorMessage}
      </p>
    {/if}

    <div class="flex min-h-0 flex-1">
      <main class="flex min-w-0 flex-1 flex-col">
        <div class="relative min-h-0 flex-1 bg-surface-0">
          {#if current.mode === "whiteboard"}
            <WhiteboardCanvas
              elements={current.whiteboardElements}
              files={current.resolvedWhiteboardFiles}
              onLocalChange={(changed) => current.pushWhiteboardElements(changed)}
              onLocalFile={(file) => void current.pushWhiteboardImage(file)}
              onLocalClear={() => current.clearWhiteboard()}
            />
          {:else if current.localScreenTrack}
            <TrackVideo track={current.localScreenTrack} />
            <span
              class="absolute top-4 left-4 flex items-center gap-2 bg-surface-0/80 px-3 py-1.5 text-sm text-ink-1"
            >
              <Icon icon="ph:broadcast-bold" width="14" class="text-accent" />
              You are sharing this screen
            </span>
            <AnnotationOverlay
              annotations={current.annotations}
              labels={participantLabels}
              interactive={isAnnotating}
              onStrokeStart={(point) => current.beginStroke(point)}
              onStrokeExtend={(point) => current.extendStroke(point)}
              onStrokeEnd={() => current.endStroke()}
              onPointerMove={(point) => current.movePointer(point)}
            />
          {:else if current.screenTrack}
            <TrackVideo track={current.screenTrack} />
            <AnnotationOverlay
              annotations={current.annotations}
              labels={participantLabels}
              interactive={isAnnotating}
              onStrokeStart={(point) => current.beginStroke(point)}
              onStrokeExtend={(point) => current.extendStroke(point)}
              onStrokeEnd={() => current.endStroke()}
              onPointerMove={(point) => current.movePointer(point)}
            />
          {:else if cameraTiles.length > 0}
            <div
              class="grid h-full gap-px p-px"
              class:grid-cols-1={cameraTiles.length === 1}
              class:grid-cols-2={cameraTiles.length > 1}
            >
              {#each cameraTiles as tile (tile.key)}
                <div class="relative min-h-0 bg-black">
                  <TrackVideo track={tile.track} />
                  <span
                    class="absolute bottom-3 left-3 flex items-center gap-2 bg-surface-0/80 px-3 py-1.5 text-sm text-ink-1"
                  >
                    <Icon icon="ph:video-camera-bold" width="14" class="text-accent" />
                    {tile.label}
                  </span>
                </div>
              {/each}
            </div>
          {:else}
            <div class="flex h-full flex-col items-center justify-center gap-4 text-center">
              <Icon icon="ph:monitor-bold" width="48" class="text-ink-2" />
              <div>
                <p class="text-lg font-medium">Nothing on screen yet</p>
                <p class="text-sm text-ink-2">
                  Share your screen, turn on a camera, or open the whiteboard to begin.
                </p>
              </div>
            </div>
          {/if}
        </div>

        {#if drawingNow.length > 0}
          <div
            class="pointer-events-none absolute bottom-20 left-4 flex items-center gap-2 bg-surface-0/85 px-3 py-2 text-sm text-ink-1"
          >
            <Icon icon="ph:pencil-simple-bold" width="14" class="text-accent" />
            {drawingNow.join(", ")}
            {drawingNow.length === 1 ? "is" : "are"} drawing
          </div>
        {/if}

        {#if isAnnotating && current.mode !== "whiteboard"}
          <AnnotationToolbar controller={current} />
        {/if}
      </main>
    </div>
  </div>
{:else}
  <div class="flex h-screen items-center justify-center">
    <Icon icon="ph:circle-notch-bold" width="32" class="animate-spin text-ink-2" />
  </div>
{/if}
