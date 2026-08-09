<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { ConnectionStatus } from "@pistation/client-core";

  let { status }: { status: ConnectionStatus } = $props();

  const presentation = {
    idle: { label: "Idle", icon: "ph:circle-bold", color: "text-ink-2" },
    connecting: { label: "Connecting", icon: "ph:circle-notch-bold", color: "text-ink-1" },
    connected: { label: "Live", icon: "ph:circle-fill", color: "text-success" },
    reconnecting: { label: "Reconnecting", icon: "ph:circle-notch-bold", color: "text-accent" },
    disconnected: { label: "Offline", icon: "ph:circle-fill", color: "text-danger" },
    failed: { label: "Failed", icon: "ph:warning-circle-bold", color: "text-danger" }
  } as const;

  const current = $derived(presentation[status]);
  const isSpinning = $derived(status === "connecting" || status === "reconnecting");
</script>

<div class="flex items-center gap-2 bg-surface-2 px-3 py-2 text-sm">
  <Icon
    icon={current.icon}
    width="12"
    class={`${current.color} ${isSpinning ? "animate-spin" : ""}`}
  />
  <span class="text-ink-1">{current.label}</span>
</div>
