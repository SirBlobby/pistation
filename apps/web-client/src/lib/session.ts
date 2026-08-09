import type { JoinResponse } from "@pistation/shared-types";
import { browser } from "$app/environment";

const SESSION_KEY = "pistation.session";
const ADMIN_KEY = "pistation.admin";

export type StoredSession = JoinResponse;

export function saveSession(session: StoredSession): void {
  if (!browser) return;
  sessionStorage.setItem(SESSION_KEY, JSON.stringify(session));
}

export function loadSession(): StoredSession | null {
  if (!browser) return null;
  const raw = sessionStorage.getItem(SESSION_KEY);
  if (!raw) return null;
  try {
    return JSON.parse(raw) as StoredSession;
  } catch {
    return null;
  }
}

export function clearSession(): void {
  if (!browser) return;
  sessionStorage.removeItem(SESSION_KEY);
}

export interface StoredAdmin {
  accessToken: string;
  email: string;
  expiresAt: number;
}

export function saveAdmin(admin: StoredAdmin): void {
  if (!browser) return;
  localStorage.setItem(ADMIN_KEY, JSON.stringify(admin));
}

export function loadAdmin(): StoredAdmin | null {
  if (!browser) return null;
  const raw = localStorage.getItem(ADMIN_KEY);
  if (!raw) return null;
  try {
    const admin = JSON.parse(raw) as StoredAdmin;
    if (admin.expiresAt <= Date.now()) {
      localStorage.removeItem(ADMIN_KEY);
      return null;
    }
    return admin;
  } catch {
    return null;
  }
}

export function clearAdmin(): void {
  if (!browser) return;
  localStorage.removeItem(ADMIN_KEY);
}
