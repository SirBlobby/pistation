import type { NightModeSettings } from "./night.js";
import type { CustomWidgetDefinition } from "./widget-builder.js";
import type { Widget } from "./widget.js";

export const KIOSK_STATUSES = ["online", "offline"] as const;

export type KioskStatus = (typeof KIOSK_STATUSES)[number];

export interface WifiMetrics {
  interface: string;
  linkQuality: number;
  signalDbm: number;
}

export interface KioskMetrics {
  cpuPercent: number | null;
  memoryUsedBytes: number;
  memoryTotalBytes: number;
  uptimeSeconds: number;
  temperatureCelsius: number | null;
  wifi: WifiMetrics | null;
}

export interface Kiosk {
  kioskId: string;
  name: string;
  location: string;
  roomName: string;
  status: KioskStatus;
  lastSeenAt: number | null;
  createdAt: number;
  metrics: KioskMetrics | null;
  metricsAt: number | null;
}

export function signalLabel(signalDbm: number): "excellent" | "good" | "weak" | "poor" {
  if (signalDbm >= -55) return "excellent";
  if (signalDbm >= -67) return "good";
  if (signalDbm >= -75) return "weak";
  return "poor";
}

/// Rough strength as a percentage. Wi-Fi signal is logarithmic and roughly spans -100 dBm
/// for unusable to -50 dBm for excellent, which is the range this maps onto.
export function signalPercent(signalDbm: number): number {
  return Math.round(Math.min(100, Math.max(0, 2 * (signalDbm + 100))));
}

export function signalAdvice(signalDbm: number): string {
  switch (signalLabel(signalDbm)) {
    case "excellent":
      return "Strong signal";
    case "good":
      return "Good signal";
    case "weak":
      return "Weak, video may stutter";
    default:
      return "Too weak, move the screen or the access point";
  }
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes / 1024;
  let unitIndex = 0;

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }

  return `${value.toFixed(value >= 10 ? 0 : 1)} ${units[unitIndex]}`;
}

export function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
}

export interface KioskBackground {
  /// Retained so layouts saved before rotation existed still resolve. The server folds a
  /// non empty value into `images` on load, so clients should read `images`.
  imageUrl: string;
  images: string[];
  rotationSeconds: number;
  fit: "cover" | "contain";
  dim: number;
}

export const DEFAULT_BACKGROUND: KioskBackground = {
  imageUrl: "",
  images: [],
  rotationSeconds: 60,
  fit: "cover",
  dim: 0.35
};

export const MIN_WALLPAPER_ROTATION_SECONDS = 5;

export const DEFAULT_WIDGET_OPACITY = 0.5;

export interface KioskLayout {
  kioskId: string;
  backgroundColor: string;
  foregroundColor: string;
  widgetOpacity: number;
  background: KioskBackground;
  nightMode: NightModeSettings;
  widgets: Widget[];
  customDefinitions: CustomWidgetDefinition[];
  updatedAt: number;
}

export function resolveMediaUrl(baseUrl: string, url: string): string {
  if (!url) return "";
  if (/^(https?:|data:|blob:)/.test(url)) return url;
  return `${baseUrl.replace(/\/$/, "")}${url.startsWith("/") ? url : `/${url}`}`;
}

export const PIN_LENGTH = 6;
export const PIN_ROTATION_SECONDS = 45;
export const PIN_GRACE_SECONDS = 15;

export function isValidPin(pin: string): boolean {
  return new RegExp(`^\\d{${PIN_LENGTH}}$`).test(pin);
}

export function formatPin(pin: string): string {
  if (pin.length !== PIN_LENGTH) return pin;
  return `${pin.slice(0, 3)} ${pin.slice(3)}`;
}
