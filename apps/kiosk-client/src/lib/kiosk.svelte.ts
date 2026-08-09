import type {
  DataEnvelope,
  KioskLayout,
  RoomMode,
  WhiteboardElement
} from "@pistation/shared-types";
import {
  isNightModeActive,
  isTopic,
  mergeWhiteboardElements,
  PIN_GRACE_SECONDS
} from "@pistation/shared-types";
import type { ConnectionStatus } from "@pistation/client-core";
import type { AnnotationState } from "@pistation/client-core/annotations";
import {
  applyAnnotationEvent,
  createAnnotationState,
  prunePointers
} from "@pistation/client-core/annotations";
import { invoke } from "@tauri-apps/api/core";

import { NativeRoom, type RoomParticipant } from "./native-room";

import {
  fetchLayout,
  fetchPin,
  fetchSession,
  KioskApiError,
  registerKiosk,
  sendHeartbeat
} from "./api";

const HEARTBEAT_INTERVAL_MS = 30_000;
const LAYOUT_INTERVAL_MS = 60_000;
const PIN_FALLBACK_INTERVAL_MS = 20_000;

interface KioskConfig {
  serverUrl: string;
  enrollmentToken: string;
  kioskToken: string;
  kioskId: string;
  roomName: string;
  livekitUrl: string;
  joinUrl: string;
}

export class KioskController {
  status = $state<ConnectionStatus>("idle");
  provisioningError = $state<string | null>(null);
  pin = $state<string | null>(null);
  layout = $state<KioskLayout | null>(null);
  mode = $state<RoomMode>("idle");
  annotations = $state<AnnotationState>(createAnnotationState());
  whiteboardElements = $state<WhiteboardElement[]>([]);
  participants = $state<RoomParticipant[]>([]);
  isScreenActive = $state(false);
  screenWidth = $state(0);
  screenHeight = $state(0);
  joinUrl = $state("");
  mediaBaseUrl = $state("");
  isNight = $state(false);
  connectionError = $state<string | null>(null);

  private config: KioskConfig | null = null;
  private connection: NativeRoom | null = null;
  private timers: ReturnType<typeof setInterval>[] = [];
  private pinTimer: ReturnType<typeof setTimeout> | null = null;
  private hasRetriedEnrollment = false;

  async start(): Promise<void> {
    try {
      this.config = await invoke<KioskConfig>("load_config");
    } catch {
      this.provisioningError = "Could not read the kiosk configuration file.";
      return;
    }

    if (!this.config.serverUrl) {
      this.provisioningError = "No server URL configured. Provision this kiosk first.";
      return;
    }

    this.joinUrl = this.config.joinUrl;
    this.mediaBaseUrl = this.config.serverUrl;

    if (!this.config.kioskToken) {
      const registered = await this.register();
      if (!registered) return;
    }

    void this.refreshPin();
    void this.refreshLayout();
    void this.connectToRoom();

    this.addInterval(() => void this.heartbeat(), HEARTBEAT_INTERVAL_MS);
    this.addInterval(() => void this.refreshLayout(), LAYOUT_INTERVAL_MS);
    this.addInterval(() => {
      this.annotations = prunePointers(this.annotations, Date.now());
    }, 1000);
    this.addInterval(() => this.evaluateNightMode(), 20_000);
  }

  async stop(): Promise<void> {
    for (const timer of this.timers) clearInterval(timer);
    this.timers = [];
    if (this.pinTimer) clearTimeout(this.pinTimer);
    this.pinTimer = null;
    await this.connection?.stop();
    this.connection = null;
  }

  private addInterval(action: () => void, intervalMs: number): void {
    this.timers.push(setInterval(action, intervalMs));
  }

  private async register(): Promise<boolean> {
    if (!this.config) return false;

    if (!this.config.enrollmentToken) {
      this.provisioningError = "This kiosk has no enrollment token. Add one in the admin panel.";
      return false;
    }

    try {
      const hardwareId = await invoke<string>("hardware_id");
      const result = await registerKiosk(
        this.config.serverUrl,
        this.config.enrollmentToken,
        hardwareId
      );

      this.config = {
        ...this.config,
        kioskId: result.kioskId,
        kioskToken: result.kioskToken,
        roomName: result.roomName,
        livekitUrl: result.livekitUrl,
        enrollmentToken: ""
      };

      await invoke("save_config", { config: this.config });
      this.provisioningError = null;
      return true;
    } catch (error) {
      this.provisioningError =
        error instanceof Error ? error.message : "Enrollment failed. Check the token.";
      return false;
    }
  }

  private async handleRejectedToken(): Promise<boolean> {
    if (!this.config) return false;

    if (this.hasRetriedEnrollment) {
      this.provisioningError =
        "This kiosk's credentials were rejected. Rotate its enrollment token in the admin panel and run the installer again.";
      return false;
    }

    this.hasRetriedEnrollment = true;
    this.config = { ...this.config, kioskToken: "" };
    await invoke("save_config", { config: this.config });

    this.config = await invoke<KioskConfig>("load_config");
    if (!(await this.register())) return false;

    void this.refreshPin();
    void this.refreshLayout();
    void this.connectToRoom();
    return true;
  }

  private async refreshPin(): Promise<void> {
    if (!this.config?.kioskToken) return;

    let delayMs = PIN_FALLBACK_INTERVAL_MS;

    try {
      const issued = await fetchPin(this.config.serverUrl, this.config.kioskToken);
      this.pin = issued.pin;
      delayMs = Math.max(5000, issued.expiresAt - Date.now() - PIN_GRACE_SECONDS * 1000);
    } catch (error) {
      this.pin = null;
      if (error instanceof KioskApiError && error.status === 401) {
        if (await this.handleRejectedToken()) return;
      }
    }

    if (this.pinTimer) clearTimeout(this.pinTimer);
    this.pinTimer = setTimeout(() => void this.refreshPin(), delayMs);
  }

  private async refreshLayout(): Promise<void> {
    if (!this.config?.kioskToken) return;

    const layout = await fetchLayout(this.config.serverUrl, this.config.kioskToken).catch(
      () => null
    );
    if (!layout) return;

    this.layout = layout;
    this.evaluateNightMode();
  }

  private evaluateNightMode(): void {
    this.isNight = this.layout ? isNightModeActive(this.layout.nightMode) : false;
  }

  private async heartbeat(): Promise<void> {
    if (!this.config?.kioskToken) return;

    const metrics = await invoke("collect_metrics").catch(() => null);
    await sendHeartbeat(this.config.serverUrl, this.config.kioskToken, metrics).catch(
      () => undefined
    );
  }

  private async connectToRoom(): Promise<void> {
    if (!this.config?.kioskToken) return;

    const config = this.config;

    this.connection = new NativeRoom(
      async () => {
        const session = await fetchSession(config.serverUrl, config.kioskToken);
        return { livekitUrl: session.livekitUrl, accessToken: session.accessToken };
      },
      {
        onStatus: (status, detail) => {
          this.status = status;

          if (status === "connected") {
            this.connectionError = null;
            return;
          }

          if (status === "disconnected" || status === "failed") {
            this.connectionError = detail;
            this.isScreenActive = false;
            this.mode = "idle";
            this.participants = [];
            this.annotations = createAnnotationState();
          }
        },
        onEnvelope: (envelope) => this.handleEnvelope(envelope),
        onVideo: (active, width, height) => {
          this.isScreenActive = active;
          this.screenWidth = width;
          this.screenHeight = height;

          if (active && this.mode === "idle") this.mode = "presentation";
          if (!active && this.mode === "presentation") this.mode = "idle";
        },
        onParticipants: (participants) => {
          this.participants = participants;
        }
      }
    );

    await this.connection.start();
  }

  private handleEnvelope(envelope: DataEnvelope): void {
    if (isTopic(envelope, "annotation")) {
      this.annotations = applyAnnotationEvent(
        this.annotations,
        envelope.payload,
        envelope.senderId
      );
      return;
    }

    if (isTopic(envelope, "control")) {
      const event = envelope.payload;
      if (event.type === "mode.set") this.mode = event.mode;
      if (event.type === "room.state") this.mode = event.state.mode;
      return;
    }

    if (isTopic(envelope, "whiteboard")) {
      const event = envelope.payload;
      if (event.type === "whiteboard.patch") {
        this.whiteboardElements = mergeWhiteboardElements(this.whiteboardElements, event.elements);
      }
      if (event.type === "whiteboard.snapshot") {
        this.whiteboardElements = event.elements;
      }
      if (event.type === "whiteboard.clear") {
        this.whiteboardElements = [];
      }
    }
  }
}
