import type { DataEnvelope, DataTopic, TopicPayloadMap } from "@pistation/shared-types";
import { decodeEnvelope, encodeEnvelope } from "@pistation/shared-types";
import { Room, RoomEvent } from "livekit-client";

export type ConnectionStatus =
  | "idle"
  | "connecting"
  | "connected"
  | "reconnecting"
  | "disconnected"
  | "failed";

export interface Credentials {
  livekitUrl: string;
  accessToken: string;
}

export interface ConnectionHandlers {
  onStatus(status: ConnectionStatus): void;
  onEnvelope(envelope: DataEnvelope): void;
  onReconnected?(): void;
  onConnectionError?(detail: string): void;
}

const RETRY_DELAYS_MS = [1000, 2000, 4000, 8000, 15000, 30000];

export function createRoom(): Room {
  return new Room({
    adaptiveStream: true,
    dynacast: true,
    disconnectOnPageLeave: true
  });
}

export function publishEnvelope<T extends DataTopic>(
  room: Room,
  topic: T,
  senderId: string,
  payload: TopicPayloadMap[T],
  reliable = true
): void {
  if (room.state !== "connected") return;
  const data = encodeEnvelope(topic, senderId, payload);
  void room.localParticipant.publishData(data, { reliable, topic });
}

export class RoomConnection {
  readonly room: Room;

  private handlers: ConnectionHandlers;
  private fetchCredentials: () => Promise<Credentials>;
  private retryIndex = 0;
  private retryTimer: ReturnType<typeof setTimeout> | null = null;
  private stopped = false;
  private visibilityListener: (() => void) | null = null;

  constructor(fetchCredentials: () => Promise<Credentials>, handlers: ConnectionHandlers) {
    this.room = createRoom();
    this.handlers = handlers;
    this.fetchCredentials = fetchCredentials;
    this.bindRoomEvents();
    this.bindVisibility();
  }

  async start(): Promise<void> {
    this.stopped = false;
    await this.attempt();
  }

  async stop(): Promise<void> {
    this.stopped = true;
    this.clearRetry();
    if (this.visibilityListener && typeof document !== "undefined") {
      document.removeEventListener("visibilitychange", this.visibilityListener);
      this.visibilityListener = null;
    }
    await this.room.disconnect();
    this.handlers.onStatus("disconnected");
  }

  private bindRoomEvents(): void {
    this.room
      .on(RoomEvent.Connected, () => {
        this.retryIndex = 0;
        this.handlers.onStatus("connected");
      })
      .on(RoomEvent.Reconnecting, () => this.handlers.onStatus("reconnecting"))
      .on(RoomEvent.Reconnected, () => {
        this.retryIndex = 0;
        this.handlers.onStatus("connected");
        this.handlers.onReconnected?.();
      })
      .on(RoomEvent.Disconnected, () => {
        if (this.stopped) return;
        this.handlers.onStatus("disconnected");
        this.scheduleRetry();
      })
      .on(RoomEvent.DataReceived, (payload: Uint8Array) => {
        const envelope = decodeEnvelope(payload);
        if (envelope) this.handlers.onEnvelope(envelope);
      });
  }

  private bindVisibility(): void {
    if (typeof document === "undefined") return;
    this.visibilityListener = () => {
      if (document.visibilityState !== "visible") return;
      if (this.stopped) return;
      if (this.room.state === "connected") return;
      this.clearRetry();
      this.retryIndex = 0;
      void this.attempt();
    };
    document.addEventListener("visibilitychange", this.visibilityListener);
  }

  private async attempt(): Promise<void> {
    if (this.stopped) return;
    this.handlers.onStatus("connecting");

    try {
      const credentials = await this.fetchCredentials();
      await this.room.connect(credentials.livekitUrl, credentials.accessToken);
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      console.error(`[pistation] livekit connection failed: ${detail}`);
      this.handlers.onConnectionError?.(detail);
      this.scheduleRetry();
    }
  }

  private scheduleRetry(): void {
    if (this.stopped || this.retryTimer) return;

    const delay = RETRY_DELAYS_MS[Math.min(this.retryIndex, RETRY_DELAYS_MS.length - 1)];
    this.retryIndex += 1;
    this.handlers.onStatus("reconnecting");

    this.retryTimer = setTimeout(() => {
      this.retryTimer = null;
      void this.attempt();
    }, delay);
  }

  private clearRetry(): void {
    if (this.retryTimer) {
      clearTimeout(this.retryTimer);
      this.retryTimer = null;
    }
  }
}
