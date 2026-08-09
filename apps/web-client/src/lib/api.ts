import type {
  AdminLoginResponse,
  CreateKioskResponse,
  JoinResponse,
  Kiosk,
  KioskDetailResponse,
  KioskListResponse,
  OrganizationBranding,
  SessionRefreshResponse
} from "@pistation/shared-types";

import { apiBaseUrl } from "./config";

export { apiBaseUrl };

export class ApiError extends Error {
  readonly status: number;
  readonly code: string;

  constructor(status: number, code: string, message: string) {
    super(message);
    this.status = status;
    this.code = code;
  }
}

interface RequestOptions {
  method?: string;
  body?: unknown;
  token?: string | null;
}

async function request<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const headers: Record<string, string> = {};
  if (options.body !== undefined) headers["content-type"] = "application/json";
  if (options.token) headers["authorization"] = `Bearer ${options.token}`;

  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl}${path}`, {
      method: options.method ?? "GET",
      headers,
      body: options.body === undefined ? undefined : JSON.stringify(options.body)
    });
  } catch {
    throw new ApiError(0, "network", "Could not reach the PiStation server.");
  }

  if (response.status === 204) return undefined as T;

  const payload = await response.json().catch(() => null);

  if (!response.ok) {
    const code = (payload as { error?: string })?.error ?? "unknown";
    const message = (payload as { message?: string })?.message ?? "Something went wrong.";
    throw new ApiError(response.status, code, message);
  }

  return payload as T;
}

export function joinRoom(pin: string, displayName: string): Promise<JoinResponse> {
  return request<JoinResponse>("/api/join", {
    method: "POST",
    body: { pin, displayName }
  });
}

export function refreshSession(sessionId: string): Promise<SessionRefreshResponse> {
  return request<SessionRefreshResponse>("/api/session/refresh", {
    method: "POST",
    body: { sessionId }
  });
}

export function adminLogin(email: string, password: string): Promise<AdminLoginResponse> {
  return request<AdminLoginResponse>("/api/admin/login", {
    method: "POST",
    body: { email, password }
  });
}

export function listKiosks(token: string): Promise<KioskListResponse> {
  return request<KioskListResponse>("/api/admin/kiosks", { token });
}

export function createKiosk(
  token: string,
  name: string,
  location: string
): Promise<CreateKioskResponse> {
  return request<CreateKioskResponse>("/api/admin/kiosks", {
    method: "POST",
    body: { name, location },
    token
  });
}

export function getKiosk(token: string, kioskId: string): Promise<KioskDetailResponse> {
  return request<KioskDetailResponse>(`/api/admin/kiosks/${kioskId}`, { token });
}

export function getBranding(): Promise<OrganizationBranding> {
  return request<OrganizationBranding>("/api/organization");
}

export function saveBranding(
  token: string,
  branding: OrganizationBranding
): Promise<OrganizationBranding> {
  return request<OrganizationBranding>("/api/admin/organization", {
    method: "PUT",
    body: { branding },
    token
  });
}

export async function uploadLogo(token: string, file: File): Promise<{ imageUrl: string }> {
  let response: Response;

  try {
    response = await fetch(`${apiBaseUrl}/api/admin/organization/logo`, {
      method: "PUT",
      headers: { "content-type": file.type, authorization: `Bearer ${token}` },
      body: file
    });
  } catch {
    throw new ApiError(0, "network", "Could not reach the PiStation server.");
  }

  const payload = await response.json().catch(() => null);

  if (!response.ok) {
    const message = (payload as { message?: string })?.message ?? "Upload failed.";
    throw new ApiError(response.status, "upload", message);
  }

  return payload as { imageUrl: string };
}

export function updateKiosk(
  token: string,
  kioskId: string,
  name: string,
  location: string
): Promise<Kiosk> {
  return request<Kiosk>(`/api/admin/kiosks/${kioskId}`, {
    method: "PATCH",
    body: { name, location },
    token
  });
}

export function deleteKiosk(token: string, kioskId: string): Promise<void> {
  return request<void>(`/api/admin/kiosks/${kioskId}`, { method: "DELETE", token });
}

export function saveKioskLayout(
  token: string,
  kioskId: string,
  layout: unknown
): Promise<unknown> {
  return request(`/api/admin/kiosks/${kioskId}/layout`, {
    method: "PUT",
    body: { layout },
    token
  });
}

export async function uploadWallpaper(
  token: string,
  kioskId: string,
  file: File
): Promise<{ imageUrl: string }> {
  let response: Response;

  try {
    response = await fetch(`${apiBaseUrl}/api/admin/kiosks/${kioskId}/wallpaper`, {
      method: "PUT",
      headers: {
        "content-type": file.type,
        authorization: `Bearer ${token}`
      },
      body: file
    });
  } catch {
    throw new ApiError(0, "network", "Could not reach the PiStation server.");
  }

  const payload = await response.json().catch(() => null);

  if (!response.ok) {
    const message = (payload as { message?: string })?.message ?? "Upload failed.";
    throw new ApiError(response.status, "upload", message);
  }

  return payload as { imageUrl: string };
}

export async function uploadKioskPackage(
  token: string,
  file: File
): Promise<{ downloadUrl: string; sizeBytes: number }> {
  let response: Response;

  try {
    response = await fetch(`${apiBaseUrl}/api/admin/packages/kiosk`, {
      method: "PUT",
      headers: {
        "content-type": "application/vnd.debian.binary-package",
        authorization: `Bearer ${token}`
      },
      body: file
    });
  } catch {
    throw new ApiError(0, "network", "Could not reach the PiStation server.");
  }

  const payload = await response.json().catch(() => null);

  if (!response.ok) {
    const message = (payload as { message?: string })?.message ?? "Upload failed.";
    throw new ApiError(response.status, "upload", message);
  }

  return payload as { downloadUrl: string; sizeBytes: number };
}

export function rotateEnrollment(
  token: string,
  kioskId: string
): Promise<{ enrollmentToken: string }> {
  return request(`/api/admin/kiosks/${kioskId}/enrollment`, { method: "POST", token });
}
