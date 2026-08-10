<script lang="ts">
  import type { WhiteboardElement, WhiteboardFile } from "@pistation/shared-types";
  import { onMount } from "svelte";

  let {
    elements,
    files = [],
    readOnly = false,
    onLocalChange,
    onLocalFile,
    onLocalClear
  }: {
    elements: WhiteboardElement[];
    files?: WhiteboardFile[];
    readOnly?: boolean;
    onLocalChange?: (changed: WhiteboardElement[]) => void;
    onLocalFile?: (file: { id: string; dataUrl: string; mimeType: string }) => void;
    onLocalClear?: () => void;
  } = $props();

  interface BinaryFile {
    id: string;
    dataURL: string;
    mimeType: string;
  }

  interface ExcalidrawApi {
    updateScene: (scene: { elements: readonly WhiteboardElement[] }) => void;
    addFiles: (files: BinaryFile[]) => void;
  }

  let container = $state<HTMLDivElement | null>(null);
  let excalidrawApi = $state<ExcalidrawApi | null>(null);
  let isDrawing = $state(false);

  const knownVersions = new Map<string, number>();
  const knownFileIds = new Set<string>();

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
          // Excalidraw's dark theme inverts the whole canvas in CSS and then cancels that
          // out per image by assigning ctx.filter. When the image lands in its cache after
          // the element is first drawn, the cancellation is missed and photos come out
          // inverted. A light board has no filter to cancel and cannot drift out of step.
          theme: "light",
          viewModeEnabled: readOnly,
          // Taken as unknown and narrowed here, so this file does not have to depend on
          // Excalidraw's exported API type just to hold a reference to it.
          excalidrawAPI: (api: unknown) => {
            excalidrawApi = api as ExcalidrawApi;
          },
          onChange: (
            sceneElements: readonly WhiteboardElement[],
            _appState: unknown,
            sceneFiles: Record<string, BinaryFile>
          ) => {
            handleSceneChange(sceneElements);
            handleSceneFiles(sceneFiles);
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

    const arrived = files.filter((file) => !knownFileIds.has(file.id));
    if (arrived.length === 0) return;

    for (const file of arrived) {
      knownFileIds.add(file.id);
    }

    void applyRemoteFiles(arrived);
  });

  $effect(() => {
    if (!excalidrawApi) return;

    // updateScene replaces the whole element set, so calling it part way through a stroke
    // throws away the points gathered so far. Nothing is applied while the pointer is
    // down; this effect runs again on release because isDrawing is reactive.
    if (isDrawing) return;

    // An emptied board has no versions to compare against, so it would never look like a
    // change and the old scene would stay on screen.
    if (elements.length === 0) {
      if (knownVersions.size === 0) return;
      knownVersions.clear();
      knownFileIds.clear();
      excalidrawApi.updateScene({ elements: [] });
      return;
    }

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

  /// Excalidraw only cancels out its dark theme canvas filter for images it has fully
  /// resolved in its own cache, so remote images are fetched and handed over as data URLs
  /// rather than as a link it has to load itself.
  async function applyRemoteFiles(arrived: WhiteboardFile[]): Promise<void> {
    const loaded = await Promise.all(arrived.map((file) => toBinaryFile(file)));
    const usable = loaded.filter((file): file is BinaryFile => file !== null);
    if (usable.length > 0) excalidrawApi?.addFiles(usable);
  }

  async function toBinaryFile(file: WhiteboardFile): Promise<BinaryFile | null> {
    if (file.url.startsWith("data:")) {
      return { id: file.id, dataURL: file.url, mimeType: file.mimeType };
    }

    try {
      const response = await fetch(file.url);
      if (!response.ok) throw new Error(`image request failed: ${response.status}`);

      const blob = await response.blob();
      const dataUrl = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(String(reader.result));
        reader.onerror = () => reject(reader.error);
        reader.readAsDataURL(blob);
      });

      return { id: file.id, dataURL: dataUrl, mimeType: blob.type || file.mimeType };
    } catch {
      knownFileIds.delete(file.id);
      return null;
    }
  }

  function handleSceneFiles(sceneFiles: Record<string, BinaryFile>): void {
    if (readOnly || !onLocalFile) return;

    for (const file of Object.values(sceneFiles ?? {})) {
      if (!file || knownFileIds.has(file.id)) continue;
      if (!file.dataURL.startsWith("data:")) continue;

      knownFileIds.add(file.id);
      onLocalFile({ id: file.id, dataUrl: file.dataURL, mimeType: file.mimeType });
    }
  }

  const pendingChanges = new Map<string, WhiteboardElement>();
  let flushTimer: ReturnType<typeof setTimeout> | null = null;

  function handleSceneChange(sceneElements: readonly WhiteboardElement[]) {
    if (readOnly) return;

    // Deleting one element leaves it in the scene marked isDeleted, which syncs like any
    // other edit. Resetting the canvas empties the array instead, so there is nothing left
    // to diff against and the other end has to be told outright.
    if (sceneElements.length === 0) {
      if (knownVersions.size === 0) return;
      knownVersions.clear();
      knownFileIds.clear();
      pendingChanges.clear();
      onLocalClear?.();
      return;
    }

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
