<script lang="ts">
  import type { LocalVideoTrack, RemoteTrack } from "livekit-client";

  let { track }: { track: RemoteTrack | LocalVideoTrack | null } = $props();

  let element = $state<HTMLVideoElement | null>(null);

  $effect(() => {
    const video = element;
    const active = track;
    if (!video || !active) return;

    active.attach(video);
    return () => {
      active.detach(video);
    };
  });
</script>

<video
  bind:this={element}
  class="h-full w-full bg-black object-contain"
  autoplay
  playsinline
  muted
></video>
