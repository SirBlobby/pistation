import type { KioskLayout, KioskPinResponse, KioskSessionResponse } from "@pistation/shared-types";

export interface RegisterResult {
  kioskId: string;
  kioskToken: string;
  roomName: string;
  livekitUrl: string;
  rotationSeconds: number;
}

export class KioskApiError extends Error {
  readonly status: number;

  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

async function request<T>(
  serverUrl: string,
  path: string,
  options: { method?: string; token?: string; body?: unknown } = {}
): Promise<T> {
  const headers: Record<string, string> = {};
  if (options.body !== undefined) headers["content-type"] = "application/json";
  if (options.token) headers["authorization"] = `Bearer ${options.token}`;

  let response: Response;

  try {
    response = await fetch(`${serverUrl.replace(/\/$/, "")}${path}`, {
      method: options.method ?? "GET",
      headers,
      body: options.body === undefined ? undefined : JSON.stringify(options.body)
    });
  } catch {
    throw new KioskApiError(0, `could not reach the server at ${serverUrl}`);
  }

  if (response.status === 204) return undefined as T;

  const payload = await response.json().catch(() => null);

  if (!response.ok) {
    const message = (payload as { message?: string })?.message ?? `request failed: ${path}`;
    throw new KioskApiError(response.status, message);
  }

  return payload as T;
}

export function registerKiosk(
  serverUrl: string,
  enrollmentToken: string,
  hardwareId: string
): Promise<RegisterResult> {
  return request<RegisterResult>(serverUrl, "/api/kiosk/register", {
    method: "POST",
    body: { enrollmentToken, hardwareId }
  });
}

export function fetchPin(serverUrl: string, token: string): Promise<KioskPinResponse> {
  return request<KioskPinResponse>(serverUrl, "/api/kiosk/pin", { token });
}

export function fetchSession(serverUrl: string, token: string): Promise<KioskSessionResponse> {
  return request<KioskSessionResponse>(serverUrl, "/api/kiosk/session", { token });
}

export function fetchLayout(serverUrl: string, token: string): Promise<KioskLayout> {
  return request<KioskLayout>(serverUrl, "/api/kiosk/layout", { token });
}

export function sendHeartbeat(
  serverUrl: string,
  token: string,
  metrics: unknown
): Promise<void> {
  return request<void>(serverUrl, "/api/kiosk/heartbeat", {
    method: "POST",
    token,
    body: { metrics }
  });
}
