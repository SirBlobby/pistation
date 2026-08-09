import type { Kiosk, KioskLayout } from "./kiosk.js";
import type { ParticipantRole } from "./room.js";

export interface ApiError {
  error: string;
  message: string;
}

export interface KioskRegisterRequest {
  enrollmentToken: string;
  hardwareId: string;
}

export interface KioskRegisterResponse {
  kioskId: string;
  kioskToken: string;
  roomName: string;
  livekitUrl: string;
  rotationSeconds: number;
}

export interface KioskPinResponse {
  pin: string;
  issuedAt: number;
  expiresAt: number;
}

export interface KioskSessionResponse {
  roomName: string;
  livekitUrl: string;
  accessToken: string;
  participantId: string;
}

export interface JoinRequest {
  pin: string;
  displayName: string;
}

export interface JoinResponse {
  sessionId: string;
  roomName: string;
  kioskName: string;
  livekitUrl: string;
  accessToken: string;
  participantId: string;
  displayName: string;
  role: ParticipantRole;
  expiresAt: number;
}

export interface SessionRefreshRequest {
  sessionId: string;
}

export interface SessionRefreshResponse {
  roomName: string;
  livekitUrl: string;
  accessToken: string;
  participantId: string;
  displayName: string;
  role: ParticipantRole;
  expiresAt: number;
}

export interface AdminLoginRequest {
  email: string;
  password: string;
}

export interface AdminLoginResponse {
  accessToken: string;
  email: string;
  expiresAt: number;
}

export interface CreateKioskRequest {
  name: string;
  location: string;
}

export interface CreateKioskResponse {
  kiosk: Kiosk;
  enrollmentToken: string;
}

export interface KioskDetailResponse {
  kiosk: Kiosk;
  layout: KioskLayout;
  currentPin: string | null;
}

export interface KioskListResponse {
  kiosks: Kiosk[];
}

export interface UpdateLayoutRequest {
  layout: KioskLayout;
}
