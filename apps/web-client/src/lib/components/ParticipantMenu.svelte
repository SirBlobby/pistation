<script lang="ts">
  import Icon from "@iconify/svelte";

  import type { RoomParticipant } from "$lib/room.svelte";

  let { participants }: { participants: RoomParticipant[] } = $props();

  let isOpen = $state(false);
  let container = $state<HTMLDivElement | null>(null);

  $effect(() => {
    if (!isOpen) return;

    const close = (event: MouseEvent) => {
      if (container && !container.contains(event.target as Node)) isOpen = false;
    };

    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  });
</script>

<div bind:this={container} class="relative">
  <button
    onclick={() => (isOpen = !isOpen)}
    aria-expanded={isOpen}
    class="flex items-center gap-2 bg-surface-2 px-3 py-2 text-sm text-ink-1 transition-colors hover:bg-surface-3"
  >
    <Icon icon="ph:users-bold" width="16" />
    {participants.length}
    <Icon icon={isOpen ? "ph:caret-up-bold" : "ph:caret-down-bold"} width="12" />
  </button>

  {#if isOpen}
    <div class="absolute top-full right-0 z-20 mt-px w-64 bg-surface-1 shadow-lg">
      <p class="bg-surface-2 px-4 py-2 text-xs tracking-wide text-ink-2 uppercase">In this room</p>

      <ul class="max-h-72 overflow-y-auto">
        {#each participants as participant (participant.participantId)}
          {@const isBroadcasting = participant.isSharing || participant.isCameraOn}
          <li class="flex items-center gap-2 px-4 py-2.5 text-sm">
            <Icon
              icon={participant.isSharing
                ? "ph:broadcast-bold"
                : participant.isCameraOn
                  ? "ph:video-camera-bold"
                  : "ph:user-bold"}
              width="16"
              class={isBroadcasting ? "shrink-0 text-accent" : "shrink-0 text-ink-2"}
            />
            <span class="min-w-0 flex-1 truncate text-ink-0">
              {participant.displayName}
            </span>
            {#if participant.isSelf}
              <span class="shrink-0 text-xs text-ink-2">you</span>
            {/if}
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</div>
