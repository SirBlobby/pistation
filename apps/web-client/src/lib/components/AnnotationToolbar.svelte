<script lang="ts">
  import Icon from "@iconify/svelte";
  import { DEFAULT_STROKE_STYLE, HIGHLIGHTER_STYLE } from "@pistation/shared-types";

  import type { RoomController } from "$lib/room.svelte";

  let { controller }: { controller: RoomController } = $props();

  const tools = [
    { id: "pen", icon: "ph:pencil-simple-bold", label: "Pen" },
    { id: "highlighter", icon: "ph:highlighter-bold", label: "Highlighter" },
    { id: "arrow", icon: "ph:arrow-up-right-bold", label: "Arrow" },
    { id: "rectangle", icon: "ph:rectangle-bold", label: "Rectangle" },
    { id: "ellipse", icon: "ph:circle-bold", label: "Ellipse" },
    { id: "laser", icon: "ph:cursor-bold", label: "Laser" }
  ] as const;

  const colors = ["#ff2d55", "#ffd60a", "#21c17a", "#4f7cff", "#f4f6f8"];

  function isToolActive(tool: string): boolean {
    return controller.tool === tool && !controller.isEraser && !controller.isPointerMode;
  }

  const thicknessValue = $derived(Math.round(controller.strokeStyle.width * 1000));

  const thicknessPreview = $derived(
    Math.max(4, Math.min(22, Math.round(controller.strokeStyle.width * 1000) + 3))
  );

  function selectTool(tool: (typeof tools)[number]["id"]) {
    controller.isEraser = false;
    controller.isPointerMode = false;
    controller.tool = tool;

    // Switching tools keeps the colour and the thickness the user picked. Only the
    // highlighter overrides them, because a thin opaque highlighter is useless.
    controller.strokeStyle =
      tool === "highlighter"
        ? { ...HIGHLIGHTER_STYLE }
        : {
            ...DEFAULT_STROKE_STYLE,
            color: controller.strokeStyle.color,
            width: controller.strokeStyle.width
          };
  }
</script>

<div class="flex flex-wrap items-center gap-1 bg-surface-2 p-2">
  {#each tools as tool}
    <button
      onclick={() => selectTool(tool.id)}
      title={tool.label}
      aria-label={tool.label}
      class="flex h-10 w-10 items-center justify-center transition-colors"
      class:bg-accent={isToolActive(tool.id)}
      class:text-white={isToolActive(tool.id)}
      class:text-ink-1={!isToolActive(tool.id)}
      class:hover:bg-surface-3={!isToolActive(tool.id)}
    >
      <Icon icon={tool.icon} width="18" />
    </button>
  {/each}

  <button
    onclick={() => {
      controller.isPointerMode = !controller.isPointerMode;
      if (controller.isPointerMode) controller.isEraser = false;
    }}
    title="Pointer, others see where you are pointing"
    aria-label="Pointer"
    class="flex h-10 w-10 items-center justify-center transition-colors"
    class:bg-accent={controller.isPointerMode}
    class:text-white={controller.isPointerMode}
    class:text-ink-1={!controller.isPointerMode}
    class:hover:bg-surface-3={!controller.isPointerMode}
  >
    <Icon icon="ph:hand-pointing-bold" width="18" />
  </button>

  <button
    onclick={() => {
      controller.isEraser = !controller.isEraser;
      if (controller.isEraser) controller.isPointerMode = false;
    }}
    title="Eraser"
    aria-label="Eraser"
    class="flex h-10 w-10 items-center justify-center transition-colors"
    class:bg-accent={controller.isEraser}
    class:text-white={controller.isEraser}
    class:text-ink-1={!controller.isEraser}
    class:hover:bg-surface-3={!controller.isEraser}
  >
    <Icon icon="ph:eraser-bold" width="18" />
  </button>

  <div class="mx-1 h-8 w-px bg-surface-3"></div>

  <label class="flex items-center gap-2 px-2" title="Line thickness">
    <span class="text-xs tracking-wide text-ink-2 uppercase">Thickness</span>
    <input
      type="range"
      min="1"
      max="30"
      value={thicknessValue}
      oninput={(event) =>
        (controller.strokeStyle = {
          ...controller.strokeStyle,
          width: Number((event.target as HTMLInputElement).value) / 1000
        })}
      class="w-28"
      aria-label="Line thickness"
    />
    <span class="w-6 text-right font-mono text-xs text-ink-1">{thicknessValue}</span>
    <span
      class="shrink-0 rounded-full bg-current"
      style={`width: ${thicknessPreview}px; height: ${thicknessPreview}px; color: ${controller.strokeStyle.color};`}
    ></span>
  </label>

  <div class="mx-1 h-8 w-px bg-surface-3"></div>

  {#each colors as color}
    <button
      onclick={() => (controller.strokeStyle = { ...controller.strokeStyle, color })}
      title={`Colour ${color}`}
      aria-label={`Colour ${color}`}
      class="h-8 w-8 transition-transform"
      class:scale-90={controller.strokeStyle.color !== color}
      style={`background-color: ${color}`}
    ></button>
  {/each}

  <div class="mx-1 h-8 w-px bg-surface-3"></div>

  <button
    onclick={() => controller.clearAnnotations()}
    title="Clear annotations"
    aria-label="Clear annotations"
    class="flex h-10 items-center gap-2 px-3 text-sm text-ink-1 transition-colors hover:bg-surface-3"
  >
    <Icon icon="ph:trash-bold" width="18" />
    Clear
  </button>
</div>
