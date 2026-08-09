export interface NightModeSettings {
  enabled: boolean;
  startTime: string;
  endTime: string;
  timeZone: string;
  showPin: boolean;
  brightness: number;
}

export const DEFAULT_NIGHT_MODE: NightModeSettings = {
  enabled: false,
  startTime: "22:00",
  endTime: "06:30",
  timeZone: "",
  showPin: true,
  brightness: 0.45
};

export function parseTimeOfDay(value: string): number | null {
  const match = /^(\d{1,2}):(\d{2})$/.exec(value.trim());
  if (!match) return null;

  const hours = Number(match[1]);
  const minutes = Number(match[2]);
  if (hours > 23 || minutes > 59) return null;

  return hours * 60 + minutes;
}

export function formatTimeOfDay(minutesOfDay: number): string {
  const normalized = ((minutesOfDay % 1440) + 1440) % 1440;
  const hours = Math.floor(normalized / 60);
  const minutes = normalized % 60;
  return `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}`;
}

export function minutesOfDayIn(timeZone: string, now: Date): number {
  if (!timeZone) return now.getHours() * 60 + now.getMinutes();

  try {
    const parts = new Intl.DateTimeFormat("en-GB", {
      timeZone,
      hour: "2-digit",
      minute: "2-digit",
      hourCycle: "h23"
    }).formatToParts(now);

    const hours = Number(parts.find((part) => part.type === "hour")?.value ?? "0");
    const minutes = Number(parts.find((part) => part.type === "minute")?.value ?? "0");
    return hours * 60 + minutes;
  } catch {
    return now.getHours() * 60 + now.getMinutes();
  }
}

export function isNightModeActive(
  settings: NightModeSettings,
  now: Date = new Date()
): boolean {
  if (!settings.enabled) return false;

  const start = parseTimeOfDay(settings.startTime);
  const end = parseTimeOfDay(settings.endTime);
  if (start === null || end === null || start === end) return false;

  const current = minutesOfDayIn(settings.timeZone, now);

  if (start < end) {
    return current >= start && current < end;
  }

  return current >= start || current < end;
}
