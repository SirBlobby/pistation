<script lang="ts">
  import Icon from "@iconify/svelte";
  import { Logo } from "@pistation/ui";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let { status }: { status: string } = $props();

  const appWindow = getCurrentWindow();

  async function toggleMaximize() {
    if (await appWindow.isMaximized()) {
      await appWindow.unmaximize();
    } else {
      await appWindow.maximize();
    }
  }
</script>

<header
  data-tauri-drag-region
  class="flex h-9 shrink-0 items-center gap-3 bg-surface-1 pr-px pl-3 select-none"
>
  <Logo size={18} />
  <span data-tauri-drag-region class="text-xs font-medium text-ink-1">PiStation kiosk</span>
  <span class="text-xs text-ink-2">{status}</span>

  <div class="ml-auto flex h-full">
    <button
      onclick={() => appWindow.minimize()}
      aria-label="Minimise"
      class="flex h-full w-11 items-center justify-center text-ink-1 transition-colors hover:bg-surface-2"
    >
      <Icon icon="ph:minus-bold" width="14" />
    </button>
    <button
      onclick={toggleMaximize}
      aria-label="Maximise"
      class="flex h-full w-11 items-center justify-center text-ink-1 transition-colors hover:bg-surface-2"
    >
      <Icon icon="ph:square-bold" width="12" />
    </button>
    <button
      onclick={() => appWindow.close()}
      aria-label="Close"
      class="flex h-full w-11 items-center justify-center text-ink-1 transition-colors hover:bg-danger hover:text-white"
    >
      <Icon icon="ph:x-bold" width="14" />
    </button>
  </div>
</header>
