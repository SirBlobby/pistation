<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { CustomWidgetDefinition, Kiosk, KioskLayout, Widget, WidgetKind } from "@pistation/shared-types";
  import {
    BUILTIN_WIDGET_KINDS,
    createId,
    DEFAULT_WIDGET_SETTINGS,
    DEFAULT_WIDGET_STYLES,
    ensureUniqueIds,
    findFreePlacement
  } from "@pistation/shared-types";
  import { NightSurface } from "@pistation/ui";
  import { onMount } from "svelte";
  import { page } from "$app/stores";

  import {
    ApiError,
    apiBaseUrl,
    getKiosk,
    rotateEnrollment,
    saveKioskLayout,
    uploadWallpaper
  } from "$lib/api";
  import AppearancePanel from "$lib/components/admin/AppearancePanel.svelte";
  import InstallCommand from "$lib/components/admin/InstallCommand.svelte";
  import KioskStats from "$lib/components/admin/KioskStats.svelte";
  import NightModePanel from "$lib/components/admin/NightModePanel.svelte";
  import WidgetBuilder from "$lib/components/admin/WidgetBuilder.svelte";
  import WidgetGridEditor from "$lib/components/admin/WidgetGridEditor.svelte";
  import WidgetList from "$lib/components/admin/WidgetList.svelte";
  import WidgetSettingsPanel from "$lib/components/admin/WidgetSettingsPanel.svelte";
  import { loadAdmin } from "$lib/session";

  const kioskId = $derived($page.params.kioskId ?? "");

  let kiosk = $state<Kiosk | null>(null);
  let layout = $state<KioskLayout | null>(null);
  let currentPin = $state<string | null>(null);
  let selectedWidgetId = $state<string | null>(null);
  let statusMessage = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let newEnrollmentToken = $state<string | null>(null);
  let isPreviewingNight = $state(false);
  let isUploading = $state(false);
  let uploadError = $state<string | null>(null);

  const admin = loadAdmin();
  const selectedWidget = $derived(
    layout?.widgets.find((widget) => widget.widgetId === selectedWidgetId) ?? null
  );
  const joinUrl = $derived(
    typeof window === "undefined" ? "" : window.location.host
  );

  onMount(() => {
    void load();

    // Only the device stats are refreshed on a timer. Re-reading the layout would throw
    // away whatever the admin is part way through editing.
    const interval = setInterval(() => void refreshStats(), 15000);
    return () => clearInterval(interval);
  });

  async function refreshStats() {
    if (!admin) return;

    try {
      const detail = await getKiosk(admin.accessToken, kioskId);
      kiosk = detail.kiosk;
      currentPin = detail.currentPin;
    } catch {
      return;
    }
  }

  async function load() {
    if (!admin) return;
    try {
      const detail = await getKiosk(admin.accessToken, kioskId);
      kiosk = detail.kiosk;
      currentPin = detail.currentPin;
      layout = {
        ...detail.layout,
        widgets: ensureUniqueIds(
          detail.layout.widgets,
          (widget) => widget.widgetId,
          (widget, widgetId) => ({ ...widget, widgetId }),
          "w"
        )
      };
    } catch (error) {
      errorMessage = error instanceof ApiError ? error.message : "Could not load this kiosk.";
    }
  }

  const PREFERRED_SPANS: Partial<Record<WidgetKind, [number, number]>> = {
    pin: [12, 4],
    clock: [5, 2],
    weather: [5, 2],
    image: [6, 4],
    agenda: [4, 4]
  };

  function addWidget(kind: WidgetKind) {
    if (!layout) return;

    const [columnSpan, rowSpan] = PREFERRED_SPANS[kind] ?? [4, 2];
    const placement = findFreePlacement(layout.widgets, columnSpan, rowSpan);

    if (!placement) {
      errorMessage = "There is no free space left on the grid. Remove or shrink a widget first.";
      return;
    }

    const widget: Widget = {
      widgetId: createId("w"),
      kind,
      placement,
      settings: structuredClone(DEFAULT_WIDGET_SETTINGS[kind]),
      style: structuredClone(DEFAULT_WIDGET_STYLES[kind]),
      enabled: true
    };

    errorMessage = null;
    layout = { ...layout, widgets: [...layout.widgets, widget] };
    selectedWidgetId = widget.widgetId;
  }

  function replaceWidgets(widgets: Widget[]) {
    if (!layout) return;
    layout = { ...layout, widgets };

    if (selectedWidgetId && !widgets.some((widget) => widget.widgetId === selectedWidgetId)) {
      selectedWidgetId = null;
    }
  }

  function updateWidget(updated: Widget) {
    if (!layout) return;
    layout = {
      ...layout,
      widgets: layout.widgets.map((widget) =>
        widget.widgetId === updated.widgetId ? updated : widget
      )
    };
  }

  function removeWidget(widgetId: string) {
    if (!layout) return;
    layout = {
      ...layout,
      widgets: layout.widgets.filter((widget) => widget.widgetId !== widgetId)
    };
    if (selectedWidgetId === widgetId) selectedWidgetId = null;
  }

  function updateDefinitions(definitions: CustomWidgetDefinition[]) {
    if (!layout) return;
    layout = { ...layout, customDefinitions: definitions };
  }

  async function save() {
    if (!admin || !layout) return;
    statusMessage = null;
    errorMessage = null;

    try {
      await saveKioskLayout(admin.accessToken, kioskId, layout);
      statusMessage = "Layout saved. The kiosk will pick it up within a minute.";
    } catch (error) {
      errorMessage = error instanceof ApiError ? error.message : "Could not save the layout.";
    }
  }

  async function handleWallpaperUpload(files: File[]) {
    if (!admin || !layout) return;

    isUploading = true;
    uploadError = null;

    const uploaded: string[] = [];

    try {
      for (const file of files) {
        const result = await uploadWallpaper(admin.accessToken, kioskId, file);
        uploaded.push(result.imageUrl);
      }
    } catch (error) {
      uploadError = error instanceof ApiError ? error.message : "Upload failed.";
    }

    if (uploaded.length > 0) {
      layout = {
        ...layout,
        background: {
          ...layout.background,
          images: [...(layout.background.images ?? []), ...uploaded]
        }
      };
      statusMessage = `Added ${uploaded.length} image${uploaded.length === 1 ? "" : "s"}. Save the layout to send it to the kiosk.`;
    }

    isUploading = false;
  }

  async function regenerateEnrollment() {
    if (!admin) return;
    if (!confirm("Rotating the enrollment token disconnects the current Pi until it re-enrolls.")) {
      return;
    }

    const response = await rotateEnrollment(admin.accessToken, kioskId);
    newEnrollmentToken = response.enrollmentToken;
  }
</script>

<svelte:head>
  <title>{kiosk ? `${kiosk.name} · Admin · PiStation` : "Kiosk · Admin · PiStation"}</title>
  <meta name="robots" content="noindex, nofollow" />
</svelte:head>

<main class="mx-auto w-full max-w-6xl px-6 py-12">
  <header class="mb-8 flex flex-wrap items-center gap-4">
    <a
      href="/admin"
      aria-label="Back to kiosks"
      class="flex h-10 w-10 items-center justify-center bg-surface-2 text-ink-1 hover:bg-surface-3"
    >
      <Icon icon="ph:arrow-left-bold" width="18" />
    </a>

    <div class="flex-1">
      <h1 class="text-xl font-semibold tracking-tight">{kiosk?.name ?? "Kiosk"}</h1>
      <p class="text-sm text-ink-2">{kiosk?.location || "No location"}</p>
    </div>

    {#if currentPin}
      <div class="bg-surface-1 px-4 py-2 text-center">
        <p class="text-xs tracking-wide text-ink-2 uppercase">Current PIN</p>
        <p class="font-mono text-lg font-semibold">{currentPin}</p>
      </div>
    {/if}

    <button
      onclick={save}
      class="flex items-center gap-2 bg-accent px-5 py-3 font-semibold text-white transition-colors hover:bg-accent-strong"
    >
      <Icon icon="ph:floppy-disk-bold" width="18" />
      Save layout
    </button>
  </header>

  {#if statusMessage}
    <p class="mb-6 flex items-center gap-2 bg-success/15 px-4 py-3 text-sm text-success">
      <Icon icon="ph:check-circle-bold" width="18" />
      {statusMessage}
    </p>
  {/if}

  {#if errorMessage}
    <p class="mb-6 flex items-center gap-2 bg-danger/15 px-4 py-3 text-sm text-danger">
      <Icon icon="ph:warning-circle-bold" width="18" />
      {errorMessage}
    </p>
  {/if}

  {#if layout}
    <div class="grid gap-6 lg:grid-cols-[minmax(0,1.6fr)_minmax(0,1fr)]">
      <section class="flex min-w-0 flex-col gap-8">
        <div class="flex flex-col gap-3">
          <div class="flex items-center gap-3">
            <h2 class="flex-1 text-sm font-semibold tracking-wide text-ink-2 uppercase">
              Preview
            </h2>
            <button
              onclick={() => (isPreviewingNight = !isPreviewingNight)}
              class="flex items-center gap-2 px-3 py-2 text-sm transition-colors"
              class:bg-accent={isPreviewingNight}
              class:text-white={isPreviewingNight}
              class:bg-surface-2={!isPreviewingNight}
              class:text-ink-1={!isPreviewingNight}
            >
              <Icon icon="ph:moon-bold" width="16" />
              Night mode
            </button>
          </div>

          {#if isPreviewingNight}
            <div class="aspect-video w-full overflow-hidden bg-surface-1">
              <NightSurface {layout} pin={currentPin} />
            </div>
          {:else}
            <WidgetGridEditor
              {layout}
              pin={currentPin}
              {joinUrl}
              mediaBaseUrl={apiBaseUrl}
              {selectedWidgetId}
              onSelect={(widgetId) => (selectedWidgetId = widgetId)}
              onWidgetsChange={replaceWidgets}
            />
            <p class="text-xs text-ink-2">
              Drag a widget to move it, drag its corner to resize. With one selected, the arrow
              keys nudge it and shift with the arrow keys resizes.
            </p>
          {/if}
        </div>

        <div class="flex flex-col gap-3">
          <h2 class="text-sm font-semibold tracking-wide text-ink-2 uppercase">Add a widget</h2>
          <div class="flex flex-wrap gap-2">
            {#each BUILTIN_WIDGET_KINDS as kind}
              <button
                onclick={() => addWidget(kind)}
                class="bg-surface-2 px-4 py-3 text-sm text-ink-1 capitalize transition-colors hover:bg-surface-3"
              >
                + {kind}
              </button>
            {/each}
          </div>
        </div>

        <WidgetBuilder definitions={layout.customDefinitions} onChange={updateDefinitions} />
      </section>

      <aside class="flex min-w-0 flex-col gap-6">
        <WidgetList
          widgets={layout.widgets}
          {selectedWidgetId}
          onSelect={(widgetId) => (selectedWidgetId = widgetId)}
          onChange={replaceWidgets}
        />

        {#if selectedWidget}
          <WidgetSettingsPanel
            widget={selectedWidget}
            definitions={layout.customDefinitions}
            layoutOpacity={layout.widgetOpacity}
            onChange={updateWidget}
            onRemove={() => removeWidget(selectedWidget.widgetId)}
          />
        {:else}
          <p class="bg-surface-1 px-5 py-8 text-center text-sm text-ink-2">
            Select a widget to edit its settings.
          </p>
        {/if}

        <div class="flex flex-col gap-4 bg-surface-1 p-5">
          <div class="flex items-center gap-2">
            <h3 class="flex-1 text-sm font-semibold tracking-wide text-ink-2 uppercase">
              Device stats
            </h3>
            <span
              class="h-2 w-2 shrink-0"
              class:bg-success={kiosk?.status === "online"}
              class:bg-ink-2={kiosk?.status !== "online"}
            ></span>
          </div>
          <KioskStats metrics={kiosk?.metrics ?? null} metricsAt={kiosk?.metricsAt ?? null} />
        </div>

        <AppearancePanel
          {layout}
          onChange={(updated) => (layout = updated)}
          onUpload={handleWallpaperUpload}
          {isUploading}
          {uploadError}
        />

        <NightModePanel {layout} onChange={(updated) => (layout = updated)} />

        {#if newEnrollmentToken}
          <InstallCommand enrollmentToken={newEnrollmentToken} kioskName={kiosk?.name ?? ""} />
        {/if}

        <div class="flex flex-col gap-3 bg-surface-1 p-5">
          <h3 class="text-sm font-semibold tracking-wide text-ink-2 uppercase">Enrollment</h3>
          <p class="text-sm text-ink-2">
            Rotating issues a fresh token and a new install command for re-imaging this Pi.
          </p>
          <button
            onclick={regenerateEnrollment}
            class="bg-surface-2 px-4 py-2 text-sm text-ink-1 transition-colors hover:bg-surface-3"
          >
            Rotate enrollment token
          </button>
        </div>
      </aside>
    </div>
  {/if}
</main>
