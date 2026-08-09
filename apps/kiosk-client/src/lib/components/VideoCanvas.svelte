<script lang="ts">
  import { Channel, invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let { visible = false }: { visible?: boolean } = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);
  let hasFrame = $state(false);

  // The channel is opened once for the life of the page. Opening it per presentation left
  // Rust holding a callback id the webview had already discarded.
  onMount(() => {
    const channel = new Channel<ArrayBuffer | number[]>();
    let disposed = false;
    let isDecoding = false;
    let pending: ImageBitmap | null = null;
    let frameRequest = 0;

    channel.onmessage = (message) => {
      // Decoding one frame at a time means a slow decode drops frames instead of
      // building a backlog, which would show up as growing latency.
      if (disposed || isDecoding) return;

      const bytes = message instanceof ArrayBuffer ? message : new Uint8Array(message);
      isDecoding = true;

      void createImageBitmap(new Blob([bytes], { type: "image/jpeg" }))
        .then((bitmap) => {
          if (disposed) {
            bitmap.close();
            return;
          }
          pending?.close();
          pending = bitmap;
        })
        .catch(() => undefined)
        .finally(() => {
          isDecoding = false;
        });
    };

    // Painting on the animation frame ties output to the display refresh, so frames are
    // never drawn twice or torn between refreshes.
    function paint() {
      frameRequest = requestAnimationFrame(paint);

      if (!pending || !canvas) return;

      const context = canvas.getContext("2d", { alpha: false });
      if (!context) return;

      if (canvas.width !== pending.width || canvas.height !== pending.height) {
        canvas.width = pending.width;
        canvas.height = pending.height;
      }

      context.drawImage(pending, 0, 0);
      pending.close();
      pending = null;
      hasFrame = true;
    }

    frameRequest = requestAnimationFrame(paint);

    void invoke("video_subscribe", { channel }).catch((error) =>
      console.error("[pistation] video_subscribe failed", error)
    );

    return () => {
      disposed = true;
      cancelAnimationFrame(frameRequest);
      pending?.close();
      void invoke("video_unsubscribe").catch(() => undefined);
    };
  });
</script>

<canvas
  bind:this={canvas}
  class="absolute inset-0 h-full w-full bg-black"
  class:hidden={!visible}
  class:opacity-0={!hasFrame}
  style="object-fit: contain; image-rendering: auto;"
></canvas>
