<script lang="ts">
  import type {
    AgendaSettings,
    ClockSettings,
    CustomSettings,
    CustomWidgetDefinition,
    ImageSettings,
    PinSettings,
    TextSettings,
    WeatherSettings,
    Widget
  } from "@pistation/shared-types";

  import AgendaWidget from "./widgets/AgendaWidget.svelte";
  import ClockWidget from "./widgets/ClockWidget.svelte";
  import CustomWidget from "./widgets/CustomWidget.svelte";
  import ImageWidget from "./widgets/ImageWidget.svelte";
  import PinWidget from "./widgets/PinWidget.svelte";
  import TextWidget from "./widgets/TextWidget.svelte";
  import WeatherWidget from "./widgets/WeatherWidget.svelte";

  let {
    widget,
    pin = null,
    joinUrl = "",
    definitions = []
  }: {
    widget: Widget;
    pin?: string | null;
    joinUrl?: string;
    definitions?: CustomWidgetDefinition[];
  } = $props();

  const definition = $derived(
    widget.kind === "custom"
      ? (definitions.find(
          (candidate) => candidate.definitionId === (widget.settings as CustomSettings).definitionId
        ) ?? null)
      : null
  );
</script>

{#if widget.kind === "clock"}
  <ClockWidget settings={widget.settings as ClockSettings} />
{:else if widget.kind === "weather"}
  <WeatherWidget settings={widget.settings as WeatherSettings} />
{:else if widget.kind === "pin"}
  <PinWidget settings={widget.settings as PinSettings} {pin} {joinUrl} />
{:else if widget.kind === "text"}
  <TextWidget settings={widget.settings as TextSettings} />
{:else if widget.kind === "image"}
  <ImageWidget settings={widget.settings as ImageSettings} />
{:else if widget.kind === "agenda"}
  <AgendaWidget settings={widget.settings as AgendaSettings} />
{:else if widget.kind === "custom"}
  <CustomWidget {definition} />
{/if}
