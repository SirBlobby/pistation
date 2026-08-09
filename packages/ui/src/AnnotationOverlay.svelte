<script lang="ts">
  import type { AnnotationState } from "@pistation/client-core/annotations";
  import { renderAnnotations } from "@pistation/client-core/annotations";
  import type { NormalizedPoint } from "@pistation/shared-types";

  let {
    annotations,
    labels,
    interactive = false,
    onStrokeStart,
    onStrokeExtend,
    onStrokeEnd,
    onPointerMove
  }: {
    annotations: AnnotationState;
    labels?: Map<string, string>;
    interactive?: boolean;
    onStrokeStart?: (point: NormalizedPoint) => void;
    onStrokeExtend?: (point: NormalizedPoint) => void;
    onStrokeEnd?: () => void;
    onPointerMove?: (point: NormalizedPoint | null) => void;
  } = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);
  let isDrawing = $state(false);

  $effect(() => {
    if (!canvas) return;

    const element = canvas;
    const observer = new ResizeObserver(() => resizeCanvas(element));
    observer.observe(element);
    resizeCanvas(element);

    return () => observer.disconnect();
  });

  $effect(() => {
    if (!canvas) return;
    const context = canvas.getContext("2d");
    if (!context) return;

    renderAnnotations({ context, width: canvas.width, height: canvas.height }, annotations, {
      labels
    });
  });

  function resizeCanvas(element: HTMLCanvasElement) {
    const rect = element.getBoundingClientRect();
    const ratio = window.devicePixelRatio || 1;
    element.width = Math.max(1, Math.round(rect.width * ratio));
    element.height = Math.max(1, Math.round(rect.height * ratio));

    const context = element.getContext("2d");
    if (context) {
      renderAnnotations({ context, width: element.width, height: element.height }, annotations, {
        labels
      });
    }
  }

  function toNormalized(event: PointerEvent): NormalizedPoint | null {
    if (!canvas) return null;
    const rect = canvas.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) return null;

    return {
      x: Math.min(1, Math.max(0, (event.clientX - rect.left) / rect.width)),
      y: Math.min(1, Math.max(0, (event.clientY - rect.top) / rect.height))
    };
  }

  function handlePointerDown(event: PointerEvent) {
    if (!interactive) return;
    const point = toNormalized(event);
    if (!point) return;

    canvas?.setPointerCapture(event.pointerId);
    isDrawing = true;
    onStrokeStart?.(point);
  }

  function handlePointerMove(event: PointerEvent) {
    if (!interactive) return;
    const point = toNormalized(event);
    if (!point) return;

    if (isDrawing) {
      onStrokeExtend?.(point);
    } else {
      onPointerMove?.(point);
    }
  }

  function handlePointerUp(event: PointerEvent) {
    if (!interactive || !isDrawing) return;
    canvas?.releasePointerCapture(event.pointerId);
    isDrawing = false;
    onStrokeEnd?.();
  }

  function handlePointerLeave() {
    if (!interactive) return;
    onPointerMove?.(null);
  }
</script>

<canvas
  bind:this={canvas}
  class="absolute inset-0 h-full w-full"
  class:pointer-events-none={!interactive}
  class:cursor-crosshair={interactive}
  style="touch-action: none;"
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
  onpointercancel={handlePointerUp}
  onpointerleave={handlePointerLeave}
></canvas>
