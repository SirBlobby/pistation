<script lang="ts">
  import type { CustomWidgetDefinition } from "@pistation/shared-types";
  import { renderTemplate, resolvePath } from "@pistation/shared-types";
  import { onMount } from "svelte";

  let { definition }: { definition: CustomWidgetDefinition | null } = $props();

  let data = $state<unknown>(null);

  async function load(source: NonNullable<CustomWidgetDefinition["dataSource"]>) {
    try {
      const response = await fetch(source.url);
      if (!response.ok) return;
      const payload = await response.json();
      data = source.rootPath ? resolvePath(payload, source.rootPath) : payload;
    } catch {
      data = null;
    }
  }

  onMount(() => {
    const source = definition?.dataSource;
    if (!source?.url) return;

    void load(source);
    const seconds = Math.max(10, source.refreshSeconds || 300);
    const interval = setInterval(() => void load(source), seconds * 1000);
    return () => clearInterval(interval);
  });

  function listItems(sourcePath: string, maxItems: number): unknown[] {
    const value = resolvePath(data, sourcePath);
    if (!Array.isArray(value)) return [];
    return value.slice(0, Math.max(1, maxItems));
  }
</script>

{#if !definition}
  <div class="flex h-full items-center justify-center text-sm opacity-40">
    Widget definition missing
  </div>
{:else}
  <div
    class="flex h-full min-w-0 flex-col gap-[3cqmin] overflow-hidden"
    style="justify-content: var(--widget-valign, center);"
  >
    {#each definition.blocks as block (block.blockId)}
      {#if block.kind === "heading"}
        <h3 class="truncate font-semibold" style="font-size: clamp(0.75rem, 12cqmin, 2.5rem);">
          {renderTemplate(block.template, data)}
        </h3>
      {:else if block.kind === "text"}
        <p class="opacity-70" style="font-size: clamp(0.65rem, 8cqmin, 1.5rem);">
          {renderTemplate(block.template, data)}
        </p>
      {:else if block.kind === "metric"}
        <div class="flex min-w-0 flex-col">
          <span
            class="truncate tracking-wide uppercase opacity-50"
            style="font-size: clamp(0.55rem, 6cqmin, 1.25rem);"
          >
            {renderTemplate(block.labelTemplate, data)}
          </span>
          <span
            class="truncate leading-none font-semibold"
            style={`color: ${definition.accentColor}; font-size: clamp(1rem, 22cqmin, 5rem);`}
          >
            {renderTemplate(block.valueTemplate, data)}{block.unit}
          </span>
        </div>
      {:else if block.kind === "list"}
        <ul class="flex min-h-0 flex-col gap-[1.5cqmin] overflow-hidden">
          {#each listItems(block.sourcePath, block.maxItems) as item}
            <li class="truncate opacity-80" style="font-size: clamp(0.6rem, 7cqmin, 1.4rem);">
              {renderTemplate(block.itemTemplate, item)}
            </li>
          {/each}
        </ul>
      {:else if block.kind === "image"}
        <img
          src={renderTemplate(block.urlTemplate, data)}
          alt=""
          class="max-h-full w-full"
          class:object-cover={block.fit === "cover"}
          class:object-contain={block.fit === "contain"}
        />
      {:else if block.kind === "divider"}
        <div class="h-px w-full bg-current opacity-20"></div>
      {/if}
    {/each}
  </div>
{/if}
