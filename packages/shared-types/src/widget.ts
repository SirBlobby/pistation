export const BUILTIN_WIDGET_KINDS = [
  "clock",
  "weather",
  "pin",
  "text",
  "image",
  "agenda",
  "custom"
] as const;

export type WidgetKind = (typeof BUILTIN_WIDGET_KINDS)[number];

export const WIDGET_GRID_COLUMNS = 12;
export const WIDGET_GRID_ROWS = 8;

export interface WidgetPlacement {
  column: number;
  row: number;
  columnSpan: number;
  rowSpan: number;
}

export interface ClockSettings {
  timeZone: string;
  showSeconds: boolean;
  showDate: boolean;
  hour12: boolean;
}

export interface WeatherSettings {
  latitude: number;
  longitude: number;
  locationLabel: string;
  units: "metric" | "imperial";
}

export interface PinSettings {
  label: string;
  showJoinUrl: boolean;
}

export interface TextSettings {
  heading: string;
  body: string;
}

export interface ImageSettings {
  imageUrl: string;
  fit: "cover" | "contain";
}

export interface AgendaSettings {
  heading: string;
  items: string[];
}

export interface CustomSettings {
  definitionId: string;
}

export interface WidgetSettingsMap {
  clock: ClockSettings;
  weather: WeatherSettings;
  pin: PinSettings;
  text: TextSettings;
  image: ImageSettings;
  agenda: AgendaSettings;
  custom: CustomSettings;
}

export const WIDGET_ALIGNMENTS = ["start", "center", "end"] as const;

export type WidgetAlign = (typeof WIDGET_ALIGNMENTS)[number];

export interface WidgetStyle {
  padding: number;
  align: WidgetAlign;
  verticalAlign: WidgetAlign;
  backgroundColor: string;
  opacity: number | null;
}

export const DEFAULT_WIDGET_STYLE: WidgetStyle = {
  padding: 5,
  align: "start",
  verticalAlign: "center",
  backgroundColor: "",
  opacity: null
};

function styleFor(overrides: Partial<WidgetStyle>): WidgetStyle {
  return { ...DEFAULT_WIDGET_STYLE, ...overrides };
}

export const DEFAULT_WIDGET_STYLES: Record<WidgetKind, WidgetStyle> = {
  clock: styleFor({}),
  weather: styleFor({}),
  pin: styleFor({ align: "center" }),
  text: styleFor({}),
  image: styleFor({ padding: 0, align: "center", opacity: 0 }),
  agenda: styleFor({}),
  custom: styleFor({})
};

export function toCssColor(hexColor: string, opacity: number): string {
  const normalized = hexColor.replace("#", "");
  if (normalized.length !== 3 && normalized.length !== 6) {
    return `rgb(255 255 255 / ${opacity})`;
  }

  const expanded =
    normalized.length === 3
      ? normalized
          .split("")
          .map((character) => character + character)
          .join("")
      : normalized;

  const red = Number.parseInt(expanded.slice(0, 2), 16);
  const green = Number.parseInt(expanded.slice(2, 4), 16);
  const blue = Number.parseInt(expanded.slice(4, 6), 16);

  return `rgb(${red} ${green} ${blue} / ${opacity})`;
}

export function toFlexAlign(align: WidgetAlign): string {
  if (align === "center") return "center";
  return align === "end" ? "flex-end" : "flex-start";
}

export function toTextAlign(align: WidgetAlign): string {
  if (align === "center") return "center";
  return align === "end" ? "right" : "left";
}

export interface Widget<K extends WidgetKind = WidgetKind> {
  widgetId: string;
  kind: K;
  placement: WidgetPlacement;
  settings: WidgetSettingsMap[K];
  style: WidgetStyle;
  enabled: boolean;
}

export const DEFAULT_WIDGET_SETTINGS: WidgetSettingsMap = {
  clock: { timeZone: "UTC", showSeconds: false, showDate: true, hour12: true },
  weather: {
    latitude: 38.8304,
    longitude: -77.3078,
    locationLabel: "Fairfax",
    units: "imperial"
  },
  pin: { label: "Join at", showJoinUrl: true },
  text: { heading: "", body: "" },
  image: { imageUrl: "", fit: "cover" },
  agenda: { heading: "Agenda", items: [] },
  custom: { definitionId: "" }
};
