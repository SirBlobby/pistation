<script lang="ts">
  import Icon from "@iconify/svelte";
  import { PIN_LENGTH } from "@pistation/shared-types";
  import { Logo } from "@pistation/ui";
  import { goto } from "$app/navigation";

  import { ApiError, joinRoom } from "$lib/api";
  import PinInput from "$lib/components/PinInput.svelte";
  import { saveSession } from "$lib/session";

  let { label = "Enter the code on screen" }: { label?: string } = $props();

  let pin = $state("");
  let displayName = $state("");
  let isJoining = $state(false);
  let errorMessage = $state<string | null>(null);

  const isReady = $derived(
    pin.replace(/\D/g, "").length === PIN_LENGTH && displayName.trim().length > 0
  );

  async function join() {
    if (!isReady || isJoining) return;

    isJoining = true;
    errorMessage = null;

    try {
      const session = await joinRoom(pin, displayName);
      saveSession(session);
      await goto("/room");
    } catch (error) {
      errorMessage =
        error instanceof ApiError ? error.message : "Could not join. Check the PIN and try again.";
      isJoining = false;
    }
  }
</script>

<div class="mx-auto flex w-full max-w-md flex-col bg-surface-1 p-6 sm:p-8">
  <Logo size={40} class="mx-auto mb-4" />
  <p class="mb-1 text-center text-sm tracking-wide text-ink-2 uppercase">{label}</p>
  <p class="mb-6 text-center text-sm text-ink-2">It rotates every minute</p>

  <PinInput bind:value={pin} onComplete={join} />

  <label class="mt-6 block">
    <span class="mb-2 block text-sm font-medium text-ink-1">
      Your name
      <span class="text-ink-2">required</span>
    </span>
    <input
      bind:value={displayName}
      placeholder="So the room knows who you are"
      maxlength="32"
      required
      class="w-full bg-surface-2 px-4 py-3 text-ink-0 placeholder:text-ink-2"
    />
  </label>

  {#if errorMessage}
    <p class="mt-4 flex items-center gap-2 bg-danger/15 px-4 py-3 text-sm text-danger">
      <Icon icon="ph:warning-circle-bold" width="18" class="shrink-0" />
      {errorMessage}
    </p>
  {/if}

  <button
    onclick={join}
    disabled={!isReady || isJoining}
    class="mt-5 flex w-full items-center justify-center gap-2 bg-accent px-6 py-4 text-base font-semibold text-white transition-colors hover:bg-accent-strong disabled:bg-surface-3 disabled:text-ink-2"
  >
    {#if isJoining}
      <Icon icon="ph:circle-notch-bold" width="20" class="animate-spin" />
      Connecting
    {:else}
      <Icon icon="ph:sign-in-bold" width="20" />
      Join screen
    {/if}
  </button>
</div>
