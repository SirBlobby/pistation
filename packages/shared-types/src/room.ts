export const ROOM_MODES = ["idle", "presentation", "whiteboard"] as const;

export type RoomMode = (typeof ROOM_MODES)[number];

export const PARTICIPANT_ROLES = ["kiosk", "presenter", "viewer", "admin"] as const;

export type ParticipantRole = (typeof PARTICIPANT_ROLES)[number];

export interface ParticipantIdentity {
  participantId: string;
  displayName: string;
  role: ParticipantRole;
}

export interface RoomState {
  roomName: string;
  mode: RoomMode;
  presenterId: string | null;
  annotationsLocked: boolean;
  updatedAt: number;
}

export function canPublishScreen(role: ParticipantRole): boolean {
  return role === "presenter" || role === "admin";
}

export function canChangeMode(role: ParticipantRole): boolean {
  return role === "presenter" || role === "admin";
}
