<script lang="ts">
  import { PIN_LENGTH } from "@pistation/shared-types";

  let { value = $bindable(""), onComplete }: {
    value?: string;
    onComplete?: (pin: string) => void;
  } = $props();

  let inputs: HTMLInputElement[] = $state([]);

  const slots = $derived(
    Array.from({ length: PIN_LENGTH }, (_, index) => value[index] ?? "")
  );

  function setDigit(index: number, digit: string) {
    const characters = value.padEnd(PIN_LENGTH, " ").split("");
    characters[index] = digit;
    value = characters.join("").trimEnd();

    if (digit && index < PIN_LENGTH - 1) {
      inputs[index + 1]?.focus();
    }

    if (value.replace(/\s/g, "").length === PIN_LENGTH) {
      onComplete?.(value);
    }

  }

  function handleInput(index: number, event: Event) {
    const target = event.target as HTMLInputElement;
    const digit = target.value.replace(/\D/g, "").slice(-1);
    target.value = digit;
    setDigit(index, digit);
  }

  function handleKeydown(index: number, event: KeyboardEvent) {
    if (event.key === "Backspace" && !slots[index] && index > 0) {
      inputs[index - 1]?.focus();
      setDigit(index - 1, "");
      event.preventDefault();
    }

    if (event.key === "ArrowLeft" && index > 0) inputs[index - 1]?.focus();
    if (event.key === "ArrowRight" && index < PIN_LENGTH - 1) inputs[index + 1]?.focus();
  }

  function handlePaste(event: ClipboardEvent) {
    const pasted = event.clipboardData?.getData("text")?.replace(/\D/g, "") ?? "";
    if (!pasted) return;
    event.preventDefault();
    value = pasted.slice(0, PIN_LENGTH);
    inputs[Math.min(value.length, PIN_LENGTH - 1)]?.focus();
    if (value.length === PIN_LENGTH) onComplete?.(value);
  }
</script>

<div class="flex w-full flex-nowrap gap-1.5 sm:gap-2" onpaste={handlePaste}>
  {#each slots as digit, index}
    <input
      bind:this={inputs[index]}
      value={digit}
      inputmode="numeric"
      autocomplete="one-time-code"
      maxlength="1"
      aria-label={`PIN digit ${index + 1}`}
      class="h-16 min-w-0 flex-1 bg-surface-2 px-0 text-center font-mono text-2xl text-ink-0 transition-colors focus:bg-surface-3 sm:h-20 sm:text-4xl"
      oninput={(event) => handleInput(index, event)}
      onkeydown={(event) => handleKeydown(index, event)}
      onfocus={(event) => (event.target as HTMLInputElement).select()}
    />
  {/each}
</div>
