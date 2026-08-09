import type { RoomMode, RoomState } from "./room.js";

export interface ModeSetEvent {
  type: "mode.set";
  mode: RoomMode;
}

export interface PresenterSetEvent {
  type: "presenter.set";
  presenterId: string | null;
}

export interface AnnotationLockEvent {
  type: "annotation.lock";
  locked: boolean;
}

export interface RoomStateEvent {
  type: "room.state";
  state: RoomState;
}

export interface RoomStateRequestEvent {
  type: "room.state.request";
}

export type ControlEvent =
  | ModeSetEvent
  | PresenterSetEvent
  | AnnotationLockEvent
  | RoomStateEvent
  | RoomStateRequestEvent;
