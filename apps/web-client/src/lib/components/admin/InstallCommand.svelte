<script lang="ts">
  import Icon from "@iconify/svelte";

  import { apiBaseUrl } from "$lib/api";

  let {
    enrollmentToken,
    kioskName = ""
  }: {
    enrollmentToken: string;
    kioskName?: string;
  } = $props();

  let hasCopied = $state(false);

  const command = $derived(
    `curl -fsSL ${apiBaseUrl}/install.sh | sudo bash -s -- --key ${enrollmentToken}`
  );

  async function copy() {
    try {
      await navigator.clipboard.writeText(command);
      hasCopied = true;
      setTimeout(() => (hasCopied = false), 2000);
    } catch {
      hasCopied = false;
    }
  }
</script>

<div class="flex min-w-0 flex-col gap-3 overflow-hidden bg-surface-1 p-5">
  <div class="flex items-center gap-2">
    <Icon icon="ph:terminal-window-bold" width="18" class="text-accent" />
    <h3 class="flex-1 text-sm font-semibold tracking-wide uppercase">
      Set up {kioskName || "this kiosk"}
    </h3>
  </div>

  <p class="text-sm text-ink-2">
    Run this once on the Raspberry Pi over SSH, with sudo as shown. It installs the kiosk,
    enlarges swap, tunes video for the board it finds, and reboots straight into the display.
    The token works only until the Pi enrolls. Needs the 64 bit Raspberry Pi OS.
  </p>

  <div class="flex min-w-0 items-stretch gap-px">
    <code
      class="min-w-0 flex-1 overflow-x-auto bg-surface-0 px-4 py-3 font-mono text-sm whitespace-pre text-accent"
      >{command}</code
    >
    <button
      onclick={copy}
      aria-label="Copy install command"
      class="flex w-12 shrink-0 items-center justify-center bg-surface-2 text-ink-1 transition-colors hover:bg-surface-3"
    >
      <Icon icon={hasCopied ? "ph:check-bold" : "ph:copy-bold"} width="18" />
    </button>
  </div>

  <p class="text-xs break-words text-ink-2">
    The Pi must be able to reach {apiBaseUrl}. Upload a kiosk build on the admin home page first,
    otherwise the script has nothing to install.
  </p>
</div>
