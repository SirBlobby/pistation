import type { ConnectionStatus } from "@pistation/client-core";
import type { DataEnvelope } from "@pistation/shared-types";
import { parseEnvelope } from "@pistation/shared-types";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

interface StatusEvent {
  status: string;
  detail: string | null;
}

interface DataEvent {
  envelope: string;
}

interface VideoEvent {
  active: boolean;
  width: number;
  height: number;
}

export interface RoomParticipant {
  identity: string;
  displayName: string;
}

interface ParticipantsEvent {
  participants: RoomParticipant[];
}

export interface Credentials {
  livekitUrl: string;
  accessToken: string;
}

export interface NativeRoomHandlers {
  onStatus(status: ConnectionStatus, detail: string | null): void;
  onEnvelope(envelope: DataEnvelope): void;
  onVideo(active: boolean, width: number, height: number): void;
  onParticipants(participants: RoomParticipant[]): void;
}

const RETRY_DELAYS_MS = [1000, 2000, 4000, 8000, 15000, 30000];

function toConnectionStatus(status: string): ConnectionStatus {
  switch (status) {
    case "connecting":
    case "connected":
    case "reconnecting":
    case "disconnected":
      return status;
    default:
      return "failed";
  }
}

/// Media lives in Rust. This drives the native LiveKit client over Tauri commands and
/// turns its events back into the same shape the web client works with.
export class NativeRoom {
  private fetchCredentials: () => Promise<Credentials>;
  private handlers: NativeRoomHandlers;
  private unlisteners: UnlistenFn[] = [];
  private retryIndex = 0;
  private retryTimer: ReturnType<typeof setTimeout> | null = null;
  private stopped = false;

  constructor(fetchCredentials: () => Promise<Credentials>, handlers: NativeRoomHandlers) {
    this.fetchCredentials = fetchCredentials;
    this.handlers = handlers;
  }

  async start(): Promise<void> {
    this.stopped = false;
    await this.bindEvents();
    await this.attempt();
  }

  async stop(): Promise<void> {
    this.stopped = true;
    this.clearRetry();

    for (const unlisten of this.unlisteners) unlisten();
    this.unlisteners = [];

    await invoke("room_disconnect").catch(() => undefined);
  }

  private async bindEvents(): Promise<void> {
    this.unlisteners.push(
      await listen<StatusEvent>("room://status", (event) => {
        const status = toConnectionStatus(event.payload.status);
        this.handlers.onStatus(status, event.payload.detail);

        if (status === "connected") {
          this.retryIndex = 0;
          return;
        }

        if (status === "disconnected" || status === "failed") {
          this.scheduleRetry();
        }
      })
    );

    this.unlisteners.push(
      await listen<DataEvent>("room://data", (event) => {
        const envelope = parseEnvelope(event.payload.envelope);
        if (envelope) this.handlers.onEnvelope(envelope);
      })
    );

    this.unlisteners.push(
      await listen<VideoEvent>("room://video", (event) => {
        this.handlers.onVideo(event.payload.active, event.payload.width, event.payload.height);
      })
    );

    this.unlisteners.push(
      await listen<ParticipantsEvent>("room://participants", (event) => {
        this.handlers.onParticipants(event.payload.participants);
      })
    );
  }

  private async attempt(): Promise<void> {
    if (this.stopped) return;

    try {
      const credentials = await this.fetchCredentials();
      await invoke("room_connect", {
        url: credentials.livekitUrl,
        token: credentials.accessToken
      });
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      console.error(`[pistation] native room connect failed: ${detail}`);
      this.handlers.onStatus("failed", detail);
      this.scheduleRetry();
    }
  }

  private scheduleRetry(): void {
    if (this.stopped || this.retryTimer) return;

    const delay = RETRY_DELAYS_MS[Math.min(this.retryIndex, RETRY_DELAYS_MS.length - 1)];
    this.retryIndex += 1;

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
