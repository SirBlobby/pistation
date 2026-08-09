<script lang="ts">
  import type { WhiteboardElement } from "@pistation/shared-types";
  import { onMount } from "svelte";

  let {
    elements,
    readOnly = false,
    onLocalChange
  }: {
    elements: WhiteboardElement[];
    readOnly?: boolean;
    onLocalChange?: (changed: WhiteboardElement[]) => void;
  } = $props();

  interface ExcalidrawApi {
    updateScene: (scene: { elements: readonly WhiteboardElement[] }) => void;
  }

  let container = $state<HTMLDivElement | null>(null);
  let excalidrawApi = $state<ExcalidrawApi | null>(null);
  let isDrawing = $state(false);

  const knownVersions = new Map<string, number>();

  onMount(() => {
    let unmount: (() => void) | null = null;

    void (async () => {
      const [reactModule, clientModule, excalidrawModule] = await Promise.all([
        import("react"),
        import("react-dom/client"),
        import("@excalidraw/excalidraw")
      ]);

      if (!container) return;

      const root = clientModule.createRoot(container);
      root.render(
        reactModule.createElement(excalidrawModule.Excalidraw, {
          theme: "dark",
          viewModeEnabled: readOnly,
          // Taken as unknown and narrowed here, so this file does not have to depend on
          // Excalidraw's exported API type just to hold a reference to it.
          excalidrawAPI: (api: unknown) => {
            excalidrawApi = api as ExcalidrawApi;
          },
          onChange: (sceneElements: readonly WhiteboardElement[]) => {
            handleSceneChange(sceneElements);
          },
          UIOptions: {
            canvasActions: {
              loadScene: false,
              saveToActiveFile: false,
              export: false,
              toggleTheme: false
            }
          }
        })
      );

      unmount = () => root.unmount();
    })();

    return () => unmount?.();
  });

  $effect(() => {
    if (!excalidrawApi) return;

    // updateScene replaces the whole element set, so calling it part way through a stroke
    // throws away the points gathered so far. Nothing is applied while the pointer is
    // down; this effect runs again on release because isDrawing is reactive.
    if (isDrawing) return;

    // Only push the scene back when something genuinely arrived from someone else.
    // Feeding our own edits back in has the same truncating effect.
    const hasRemoteChange = elements.some((element) => {
      const known = knownVersions.get(element.id);
      return known === undefined || element.version > known;
    });

    if (!hasRemoteChange) return;

    for (const element of elements) {
      knownVersions.set(element.id, element.version);
    }

    excalidrawApi.updateScene({ elements });
  });

  const pendingChanges = new Map<string, WhiteboardElement>();
  let flushTimer: ReturnType<typeof setTimeout> | null = null;

  function handleSceneChange(sceneElements: readonly WhiteboardElement[]) {
    if (readOnly) return;

    for (const element of sceneElements) {
      const known = knownVersions.get(element.id);
      if (known === undefined || element.version > known) {
        knownVersions.set(element.id, element.version);
        pendingChanges.set(element.id, element);
      }
    }

    if (pendingChanges.size === 0 || flushTimer) return;

    // Excalidraw reports a change on every pointer move. Batching keeps the data channel
    // usable on a Pi without making strokes feel laggy.
    flushTimer = setTimeout(() => {
      flushTimer = null;
      const changed = [...pendingChanges.values()];
      pendingChanges.clear();
      if (changed.length > 0) onLocalChange?.(changed);
    }, 80);
  }
</script>

<div
  bind:this={container}
  class="h-full w-full bg-surface-1"
  onpointerdowncapture={() => (isDrawing = true)}
  onpointerupcapture={() => (isDrawing = false)}
  onpointercancelcapture={() => (isDrawing = false)}
></div>
