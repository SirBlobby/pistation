<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { CustomWidgetDefinition, WidgetBlock } from "@pistation/shared-types";
  import { createEmptyDefinition, createId, WIDGET_BLOCK_KINDS } from "@pistation/shared-types";

  let {
    definitions,
    onChange
  }: {
    definitions: CustomWidgetDefinition[];
    onChange: (definitions: CustomWidgetDefinition[]) => void;
  } = $props();

  let selectedId = $state<string | null>(null);

  const selected = $derived(
    definitions.find((definition) => definition.definitionId === selectedId) ?? null
  );

  const makeId = createId;

  function addDefinition() {
    const definition = createEmptyDefinition(makeId("def"));
    onChange([...definitions, definition]);
    selectedId = definition.definitionId;
  }

  function updateDefinition(patch: Partial<CustomWidgetDefinition>) {
    if (!selected) return;
    onChange(
      definitions.map((definition) =>
        definition.definitionId === selected.definitionId
          ? { ...definition, ...patch, updatedAt: Date.now() }
          : definition
      )
    );
  }

  function removeDefinition(definitionId: string) {
    onChange(definitions.filter((definition) => definition.definitionId !== definitionId));
    if (selectedId === definitionId) selectedId = null;
  }

  function addBlock(kind: (typeof WIDGET_BLOCK_KINDS)[number]) {
    if (!selected) return;
    const blockId = makeId("block");

    const block = {
      heading: { blockId, kind: "heading", template: "Title" },
      text: { blockId, kind: "text", template: "Some text" },
      metric: { blockId, kind: "metric", labelTemplate: "Label", valueTemplate: "{{value}}", unit: "" },
      list: { blockId, kind: "list", sourcePath: "items", itemTemplate: "{{name}}", maxItems: 5 },
      image: { blockId, kind: "image", urlTemplate: "", fit: "contain" },
      divider: { blockId, kind: "divider" }
    }[kind] as WidgetBlock;

    updateDefinition({ blocks: [...selected.blocks, block] });
  }

  function updateBlock(blockId: string, patch: Record<string, unknown>) {
    if (!selected) return;
    updateDefinition({
      blocks: selected.blocks.map((block) =>
        block.blockId === blockId ? ({ ...block, ...patch } as WidgetBlock) : block
      )
    });
  }

  function removeBlock(blockId: string) {
    if (!selected) return;
    updateDefinition({ blocks: selected.blocks.filter((block) => block.blockId !== blockId) });
  }

  function moveBlock(index: number, offset: number) {
    if (!selected) return;
    const target = index + offset;
    if (target < 0 || target >= selected.blocks.length) return;

    const blocks = [...selected.blocks];
    const [moved] = blocks.splice(index, 1);
    blocks.splice(target, 0, moved);
    updateDefinition({ blocks });
  }
</script>

<div class="flex flex-col gap-4">
  <div class="flex items-center gap-2">
    <h3 class="flex-1 text-sm font-semibold tracking-wide text-ink-2 uppercase">Widget builder</h3>
    <button
      onclick={addDefinition}
      class="flex items-center gap-2 bg-surface-2 px-3 py-2 text-sm text-ink-1 hover:bg-surface-3"
    >
      <Icon icon="ph:plus-bold" width="16" />
      New
    </button>
  </div>

  <div class="flex flex-wrap gap-px">
    {#each definitions as definition (definition.definitionId)}
      <button
        onclick={() => (selectedId = definition.definitionId)}
        class="flex items-center gap-2 px-4 py-2 text-sm transition-colors"
        class:bg-accent={selectedId === definition.definitionId}
        class:text-white={selectedId === definition.definitionId}
        class:bg-surface-2={selectedId !== definition.definitionId}
        class:text-ink-1={selectedId !== definition.definitionId}
      >
        {definition.name}
      </button>
    {/each}
  </div>

  {#if selected}
    <div class="flex flex-col gap-4 bg-surface-1 p-5">
      <div class="flex flex-wrap gap-3">
        <label class="min-w-40 flex-1">
          <span class="mb-1 block text-xs text-ink-2">Name</span>
          <input
            value={selected.name}
            oninput={(event) =>
              updateDefinition({ name: (event.target as HTMLInputElement).value })}
            class="w-full bg-surface-2 px-3 py-2 text-sm"
          />
        </label>
        <label>
          <span class="mb-1 block text-xs text-ink-2">Accent</span>
          <input
            type="color"
            value={selected.accentColor}
            oninput={(event) =>
              updateDefinition({ accentColor: (event.target as HTMLInputElement).value })}
            class="h-10 w-16 bg-surface-2"
          />
        </label>
        <button
          onclick={() => removeDefinition(selected.definitionId)}
          class="self-end bg-surface-2 px-4 py-2 text-sm text-danger hover:bg-surface-3"
        >
          Delete
        </button>
      </div>

      <div class="flex flex-col gap-3 bg-surface-2 p-4">
        <p class="text-xs tracking-wide text-ink-2 uppercase">Data source</p>
        <input
          value={selected.dataSource?.url ?? ""}
          placeholder="https://example.com/api.json"
          oninput={(event) =>
            updateDefinition({
              dataSource: {
                url: (event.target as HTMLInputElement).value,
                refreshSeconds: selected.dataSource?.refreshSeconds ?? 300,
                rootPath: selected.dataSource?.rootPath ?? ""
              }
            })}
          class="w-full bg-surface-1 px-3 py-2 text-sm"
        />
        <div class="grid grid-cols-2 gap-3">
          <label>
            <span class="mb-1 block text-xs text-ink-2">Refresh seconds</span>
            <input
              type="number"
              min="10"
              value={selected.dataSource?.refreshSeconds ?? 300}
              oninput={(event) =>
                updateDefinition({
                  dataSource: {
                    url: selected.dataSource?.url ?? "",
                    refreshSeconds: Number((event.target as HTMLInputElement).value),
                    rootPath: selected.dataSource?.rootPath ?? ""
                  }
                })}
              class="w-full bg-surface-1 px-3 py-2 text-sm"
            />
          </label>
          <label>
            <span class="mb-1 block text-xs text-ink-2">Root path</span>
            <input
              value={selected.dataSource?.rootPath ?? ""}
              placeholder="data.current"
              oninput={(event) =>
                updateDefinition({
                  dataSource: {
                    url: selected.dataSource?.url ?? "",
                    refreshSeconds: selected.dataSource?.refreshSeconds ?? 300,
                    rootPath: (event.target as HTMLInputElement).value
                  }
                })}
              class="w-full bg-surface-1 px-3 py-2 text-sm"
            />
          </label>
        </div>
        <p class="text-xs text-ink-2">
          Reference fields in any template with double braces, for example
          <code class="text-accent">{"{{temperature}}"}</code>.
        </p>
      </div>

      <div class="flex flex-wrap gap-2">
        {#each WIDGET_BLOCK_KINDS as kind}
          <button
            onclick={() => addBlock(kind)}
            class="bg-surface-2 px-3 py-2 text-sm text-ink-1 capitalize hover:bg-surface-3"
          >
            + {kind}
          </button>
        {/each}
      </div>

      <div class="flex flex-col gap-px">
        {#each selected.blocks as block, index (block.blockId)}
          <div class="flex flex-col gap-2 bg-surface-2 p-4">
            <div class="flex items-center gap-2">
              <span class="flex-1 text-xs tracking-wide text-ink-2 uppercase">{block.kind}</span>
              <button
                onclick={() => moveBlock(index, -1)}
                aria-label="Move up"
                class="flex h-7 w-7 items-center justify-center bg-surface-1 text-ink-1 hover:bg-surface-3"
              >
                <Icon icon="ph:arrow-up-bold" width="14" />
              </button>
              <button
                onclick={() => moveBlock(index, 1)}
                aria-label="Move down"
                class="flex h-7 w-7 items-center justify-center bg-surface-1 text-ink-1 hover:bg-surface-3"
              >
                <Icon icon="ph:arrow-down-bold" width="14" />
              </button>
              <button
                onclick={() => removeBlock(block.blockId)}
                aria-label="Remove block"
                class="flex h-7 w-7 items-center justify-center bg-surface-1 text-danger hover:bg-surface-3"
              >
                <Icon icon="ph:x-bold" width="14" />
              </button>
            </div>

            {#if block.kind === "heading" || block.kind === "text"}
              <input
                value={block.template}
                oninput={(event) =>
                  updateBlock(block.blockId, {
                    template: (event.target as HTMLInputElement).value
                  })}
                class="w-full bg-surface-1 px-3 py-2 text-sm"
              />
            {:else if block.kind === "metric"}
              <div class="grid grid-cols-3 gap-2">
                <input
                  value={block.labelTemplate}
                  placeholder="Label"
                  oninput={(event) =>
                    updateBlock(block.blockId, {
                      labelTemplate: (event.target as HTMLInputElement).value
                    })}
                  class="bg-surface-1 px-3 py-2 text-sm"
                />
                <input
                  value={block.valueTemplate}
                  placeholder={"{{value}}"}
                  oninput={(event) =>
                    updateBlock(block.blockId, {
                      valueTemplate: (event.target as HTMLInputElement).value
                    })}
                  class="bg-surface-1 px-3 py-2 text-sm"
                />
                <input
                  value={block.unit}
                  placeholder="Unit"
                  oninput={(event) =>
                    updateBlock(block.blockId, { unit: (event.target as HTMLInputElement).value })}
                  class="bg-surface-1 px-3 py-2 text-sm"
                />
              </div>
            {:else if block.kind === "list"}
              <div class="grid grid-cols-3 gap-2">
                <input
                  value={block.sourcePath}
                  placeholder="items"
                  oninput={(event) =>
                    updateBlock(block.blockId, {
                      sourcePath: (event.target as HTMLInputElement).value
                    })}
                  class="bg-surface-1 px-3 py-2 text-sm"
                />
                <input
                  value={block.itemTemplate}
                  placeholder={"{{name}}"}
                  oninput={(event) =>
                    updateBlock(block.blockId, {
                      itemTemplate: (event.target as HTMLInputElement).value
                    })}
                  class="bg-surface-1 px-3 py-2 text-sm"
                />
                <input
                  type="number"
                  min="1"
                  value={block.maxItems}
                  oninput={(event) =>
                    updateBlock(block.blockId, {
                      maxItems: Number((event.target as HTMLInputElement).value)
                    })}
                  class="bg-surface-1 px-3 py-2 text-sm"
                />
              </div>
            {:else if block.kind === "image"}
              <input
                value={block.urlTemplate}
                placeholder={"https://example.com/{{path}}"}
                oninput={(event) =>
                  updateBlock(block.blockId, {
                    urlTemplate: (event.target as HTMLInputElement).value
                  })}
                class="w-full bg-surface-1 px-3 py-2 text-sm"
              />
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
