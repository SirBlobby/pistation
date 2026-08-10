import type {
  ControlEvent,
  DataEnvelope,
  DataTopic,
  NormalizedPoint,
  RoomMode,
  StrokeStyle,
  TopicPayloadMap,
  WhiteboardElement,
  WhiteboardEvent,
  WhiteboardFile
} from "@pistation/shared-types";
import {
  DEFAULT_STROKE_STYLE,
  isTopic,
  mergeWhiteboardElements,
  mergeWhiteboardFiles
} from "@pistation/shared-types";
import type { AnnotationState, ConnectionStatus } from "@pistation/client-core";
import {
  applyAnnotationEvent,
  createAnnotationState,
  eraseAtPoint,
  makeStrokeId,
  prunePointers,
  publishEnvelope,
  RoomConnection
} from "@pistation/client-core";
import {
  LocalTrackPublication,
  LocalVideoTrack,
  RemoteParticipant,
  RemoteTrack,
  RemoteTrackPublication,
  RoomEvent,
  Track
} from "livekit-client";

import { apiBaseUrl, refreshSession, uploadWhiteboardImage } from "./api";
import type { StoredSession } from "./session";

export type AnnotationTool = "pen" | "highlighter" | "arrow" | "rectangle" | "ellipse" | "laser";

export interface RoomParticipant {
  participantId: string;
  displayName: string;
  isSelf: boolean;
  isSharing: boolean;
  isCameraOn: boolean;
}

export interface CameraFeed {
  participantId: string;
  displayName: string;
  track: RemoteTrack;
}

export class RoomController {
  status = $state<ConnectionStatus>("idle");
  mode = $state<RoomMode>("idle");
  annotations = $state<AnnotationState>(createAnnotationState());
  whiteboardElements = $state<WhiteboardElement[]>([]);
  whiteboardFiles = $state<WhiteboardFile[]>([]);
  whiteboardOwnerId = $state<string | null>(null);
  screenTrack = $state<RemoteTrack | null>(null);
  localScreenTrack = $state<LocalVideoTrack | null>(null);
  localCameraTrack = $state<LocalVideoTrack | null>(null);
  cameraFeeds = $state<CameraFeed[]>([]);
  isSharing = $state(false);
  isCameraOn = $state(false);
  facingMode = $state<"user" | "environment">("environment");
  participants = $state<RoomParticipant[]>([]);
  sharingParticipantName = $state<string | null>(null);
  errorMessage = $state<string | null>(null);

  tool = $state<AnnotationTool>("pen");
  strokeStyle = $state<StrokeStyle>({ ...DEFAULT_STROKE_STYLE });
  isEraser = $state(false);
  isPointerMode = $state(false);

  private session: StoredSession;
  private connection: RoomConnection | null = null;
  private activeStrokeId: string | null = null;
  private pendingPoints: NormalizedPoint[] = [];
  private flushTimer: ReturnType<typeof setInterval> | null = null;

  constructor(session: StoredSession) {
    this.session = session;
  }

  get canPresent(): boolean {
    return this.session.role !== "viewer";
  }

  get participantId(): string {
    return this.session.participantId;
  }

  get kioskName(): string {
    return this.session.kioskName;
  }

  get role(): string {
    return this.session.role;
  }

  async connect(): Promise<void> {
    this.connection = new RoomConnection(
      async () => {
        const refreshed = await refreshSession(this.session.sessionId);
        return {
          livekitUrl: this.session.livekitUrl,
          accessToken: refreshed.accessToken
        };
      },
      {
        onStatus: (status) => {
          this.status = status;
        },
        onEnvelope: (envelope) => this.handleEnvelope(envelope),
        onReconnected: () => this.requestRoomState()
      }
    );

    this.bindTrackEvents();
    await this.connection.start();
    this.requestRoomState();
    this.startPointerPruning();
  }

  async disconnect(): Promise<void> {
    if (this.flushTimer) clearInterval(this.flushTimer);
    this.flushTimer = null;
    await this.connection?.stop();
    this.connection = null;
  }

  private bindTrackEvents(): void {
    const room = this.connection?.room;
    if (!room) return;

    room
      .on(
        RoomEvent.TrackSubscribed,
        (
          track: RemoteTrack,
          publication: RemoteTrackPublication,
          participant: RemoteParticipant
        ) => {
          if (track.kind !== Track.Kind.Video) return;

          if (publication.source === Track.Source.ScreenShare) {
            this.screenTrack = track;
          }

          if (publication.source === Track.Source.Camera) {
            this.cameraFeeds = [
              ...this.cameraFeeds.filter((feed) => feed.track !== track),
              {
                participantId: participant.identity,
                displayName: participant.name || participant.identity,
                track
              }
            ];
          }

          this.refreshParticipants();
        }
      )
      .on(RoomEvent.TrackUnsubscribed, (track: RemoteTrack) => {
        if (this.screenTrack === track) this.screenTrack = null;
        this.cameraFeeds = this.cameraFeeds.filter((feed) => feed.track !== track);
        this.refreshParticipants();
      })
      .on(RoomEvent.LocalTrackPublished, (publication: LocalTrackPublication) => {
        if (publication.source === Track.Source.ScreenShare) {
          this.isSharing = true;
          this.localScreenTrack = (publication.track as LocalVideoTrack) ?? null;
        }

        if (publication.source === Track.Source.Camera) {
          this.isCameraOn = true;
          this.localCameraTrack = (publication.track as LocalVideoTrack) ?? null;
        }

        this.refreshParticipants();
      })
      .on(RoomEvent.LocalTrackUnpublished, (publication: LocalTrackPublication) => {
        if (publication.source === Track.Source.ScreenShare) {
          this.isSharing = false;
          this.localScreenTrack = null;
        }

        if (publication.source === Track.Source.Camera) {
          this.isCameraOn = false;
          this.localCameraTrack = null;
        }

        this.refreshParticipants();
      })
      .on(RoomEvent.ParticipantConnected, () => this.refreshParticipants())
      .on(RoomEvent.ParticipantDisconnected, () => this.refreshParticipants())
      .on(RoomEvent.Connected, () => this.refreshParticipants());
  }

  private refreshParticipants(): void {
    const room = this.connection?.room;
    if (!room) {
      this.participants = [];
      this.sharingParticipantName = null;
      return;
    }

    const publishes = (
      participant: { trackPublications: Map<string, { source: Track.Source }> },
      source: Track.Source
    ) =>
      [...participant.trackPublications.values()].some(
        (publication) => publication.source === source
      );

    const self: RoomParticipant = {
      participantId: this.session.participantId,
      displayName: this.session.displayName,
      isSelf: true,
      isSharing: this.isSharing,
      isCameraOn: this.isCameraOn
    };

    const others = [...room.remoteParticipants.values()].map((participant) => ({
      participantId: participant.identity,
      displayName: participant.name || participant.identity,
      isSelf: false,
      isSharing: publishes(participant, Track.Source.ScreenShare),
      isCameraOn: publishes(participant, Track.Source.Camera)
    }));

    this.participants = [self, ...others];
    this.sharingParticipantName = this.broadcaster?.displayName ?? null;

    // Whoever opened the whiteboard has gone, so it would otherwise be stuck open for
    // everyone still in the room.
    if (
      this.whiteboardOwnerId !== null &&
      !this.participants.some(
        (participant) => participant.participantId === this.whiteboardOwnerId
      )
    ) {
      this.whiteboardOwnerId = null;
    }
  }

  /// The room has a single broadcast slot. Whoever holds it, with either a screen or a
  /// camera, holds it until they stop.
  get broadcaster(): RoomParticipant | null {
    return (
      this.participants.find((participant) => participant.isSharing || participant.isCameraOn) ??
      null
    );
  }

  get isSomeoneElseSharing(): boolean {
    const holder = this.broadcaster;
    return Boolean(holder && !holder.isSelf);
  }

  /// One source at a time, per room and per person, so a phone camera cannot quietly ride
  /// alongside a screen share.
  get canStartScreenShare(): boolean {
    return !this.isSomeoneElseSharing && !this.isCameraOn;
  }

  get canStartCamera(): boolean {
    return !this.isSomeoneElseSharing && !this.isSharing;
  }

  get blockedReason(): string | null {
    const holder = this.broadcaster;
    if (holder && !holder.isSelf) {
      const what = holder.isSharing ? "sharing a screen" : "sharing a camera";
      return `${holder.displayName} is ${what}`;
    }
    return null;
  }

  get participantCount(): number {
    return this.participants.length;
  }

  async startScreenShare(): Promise<void> {
    const room = this.connection?.room;
    if (!room) return;

    if (!this.canStartScreenShare) {
      this.errorMessage =
        this.blockedReason ?? "Turn your camera off before sharing your screen.";
      return;
    }

    try {
      this.errorMessage = null;
      await room.localParticipant.setScreenShareEnabled(true, {
        audio: false,
        contentHint: "detail"
      });
      this.setMode("presentation");
    } catch {
      this.errorMessage = "Screen sharing was blocked or cancelled.";
    }
  }

  /// Publishes the device camera. Phones default to the rear lens, which is what people
  /// point at a whiteboard or a room, and can be flipped without dropping the track.
  async startCamera(): Promise<void> {
    const room = this.connection?.room;
    if (!room) return;

    if (!this.canStartCamera) {
      this.errorMessage =
        this.blockedReason ?? "Stop sharing your screen before turning the camera on.";
      return;
    }

    try {
      this.errorMessage = null;
      await room.localParticipant.setCameraEnabled(true, {
        facingMode: this.facingMode,
        resolution: { width: 1280, height: 720 }
      });
    } catch {
      this.errorMessage = "Could not use the camera. Check the site has permission.";
      this.isCameraOn = false;
    }
  }

  async stopCamera(): Promise<void> {
    const room = this.connection?.room;
    if (!room) return;
    await room.localParticipant.setCameraEnabled(false);
  }

  async flipCamera(): Promise<void> {
    this.facingMode = this.facingMode === "environment" ? "user" : "environment";
    if (!this.isCameraOn) return;

    const track = this.localCameraTrack;
    if (!track) return;

    try {
      await track.restartTrack({ facingMode: this.facingMode });
    } catch {
      this.errorMessage = "This device only has one camera.";
    }
  }

  async stopScreenShare(): Promise<void> {
    const room = this.connection?.room;
    if (!room) return;
    await room.localParticipant.setScreenShareEnabled(false);
    this.setMode("idle");
  }

  setMode(mode: RoomMode): void {
    if (!this.canPresent) return;
    if (this.mode === "whiteboard" && mode !== "whiteboard" && !this.canStopWhiteboard) return;

    const ownerId = mode === "whiteboard" ? this.session.participantId : null;
    this.whiteboardOwnerId = ownerId;
    this.mode = mode;
    this.send("control", { type: "mode.set", mode, ownerId });
  }

  /// A whiteboard belongs to whoever opened it, so nobody else can close it out from under
  /// them. If that person has left the room the board is unowned and anyone may close it.
  get canStopWhiteboard(): boolean {
    if (this.whiteboardOwnerId === null) return true;
    return this.whiteboardOwnerId === this.session.participantId;
  }

  get whiteboardOwnerName(): string | null {
    if (this.whiteboardOwnerId === null) return null;
    const owner = this.participants.find(
      (participant) => participant.participantId === this.whiteboardOwnerId
    );
    return owner?.displayName ?? null;
  }

  beginStroke(point: NormalizedPoint): void {
    if (this.isPointerMode) {
      this.movePointer(point);
      return;
    }

    if (this.isEraser) {
      this.eraseAt(point);
      return;
    }

    const strokeId = makeStrokeId();
    this.activeStrokeId = strokeId;
    this.pendingPoints = [];

    const event = {
      type: "stroke.start" as const,
      strokeId,
      tool: this.tool,
      style: { ...this.strokeStyle },
      point
    };

    this.annotations = applyAnnotationEvent(this.annotations, event, this.session.participantId);
    this.send("annotation", event);
  }

  extendStroke(point: NormalizedPoint): void {
    if (this.isPointerMode) {
      this.movePointer(point);
      return;
    }

    if (this.isEraser) {
      this.eraseAt(point);
      return;
    }
    if (!this.activeStrokeId) return;

    const event = {
      type: "stroke.append" as const,
      strokeId: this.activeStrokeId,
      points: [point]
    };

    this.annotations = applyAnnotationEvent(this.annotations, event, this.session.participantId);
    this.pendingPoints.push(point);
    this.flushPendingPoints();
  }

  endStroke(): void {
    if (this.isPointerMode) return;
    if (!this.activeStrokeId) return;
    const strokeId = this.activeStrokeId;
    this.activeStrokeId = null;
    this.flushPendingPoints(true);

    const event = { type: "stroke.end" as const, strokeId };
    this.annotations = applyAnnotationEvent(this.annotations, event, this.session.participantId);
    this.send("annotation", event);
  }

  movePointer(point: NormalizedPoint | null): void {
    this.send(
      "annotation",
      { type: "pointer.move", point, color: this.strokeStyle.color },
      false
    );
  }

  clearAnnotations(): void {
    const event = { type: "canvas.clear" as const };
    this.annotations = applyAnnotationEvent(this.annotations, event, this.session.participantId);
    this.send("annotation", event);
  }

  pushWhiteboardElements(elements: WhiteboardElement[]): void {
    if (elements.length === 0) return;
    this.whiteboardElements = mergeWhiteboardElements(this.whiteboardElements, elements);
    this.send("whiteboard", { type: "whiteboard.patch", elements });
  }

  /// Image bytes are far too large for a data channel packet, so the binary goes to the
  /// server over HTTP and only the URL it returns travels to the other participants.
  async pushWhiteboardImage(file: {
    id: string;
    dataUrl: string;
    mimeType: string;
  }): Promise<void> {
    try {
      const blob = await (await fetch(file.dataUrl)).blob();
      const { imageUrl } = await uploadWhiteboardImage(this.session.sessionId, blob);

      const shared: WhiteboardFile = { id: file.id, url: imageUrl, mimeType: file.mimeType };
      this.whiteboardFiles = mergeWhiteboardFiles(this.whiteboardFiles, [shared]);
      this.send("whiteboard", { type: "whiteboard.patch", elements: [], files: [shared] });
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      this.errorMessage = `That image could not be shared: ${detail}`;
    }
  }

  clearWhiteboard(): void {
    this.whiteboardElements = [];
    this.whiteboardFiles = [];
    this.send("whiteboard", { type: "whiteboard.clear" });
  }

  get resolvedWhiteboardFiles(): WhiteboardFile[] {
    return this.whiteboardFiles.map((file) => ({
      ...file,
      url: file.url.startsWith("http") ? file.url : `${apiBaseUrl}${file.url}`
    }));
  }

  private eraseAt(point: NormalizedPoint): void {
    const strokeIds = eraseAtPoint(this.annotations, point, 0.02);
    if (strokeIds.length === 0) return;

    const event = { type: "stroke.erase" as const, strokeIds };
    this.annotations = applyAnnotationEvent(this.annotations, event, this.session.participantId);
    this.send("annotation", event);
  }

  private flushPendingPoints(force = false): void {
    if (this.pendingPoints.length === 0) return;
    if (!force && this.pendingPoints.length < 3) return;
    if (!this.activeStrokeId && !force) return;

    const strokeId = this.activeStrokeId;
    const points = this.pendingPoints;
    this.pendingPoints = [];
    if (!strokeId) return;

    this.send("annotation", { type: "stroke.append", strokeId, points }, false);
  }

  private requestRoomState(): void {
    this.send("control", { type: "room.state.request" });
    this.send("whiteboard", { type: "whiteboard.request" });
  }

  private startPointerPruning(): void {
    this.flushTimer = setInterval(() => {
      this.annotations = prunePointers(this.annotations, Date.now());
    }, 1000);
  }

  private send<T extends DataTopic>(
    topic: T,
    payload: TopicPayloadMap[T],
    reliable = true
  ): void {
    const room = this.connection?.room;
    if (!room) return;
    publishEnvelope(room, topic, this.session.participantId, payload, reliable);
  }

  private handleEnvelope(envelope: DataEnvelope): void {
    if (envelope.senderId === this.session.participantId) return;

    if (isTopic(envelope, "annotation")) {
      this.annotations = applyAnnotationEvent(
        this.annotations,
        envelope.payload,
        envelope.senderId
      );
      return;
    }

    if (isTopic(envelope, "control")) {
      this.handleControlEvent(envelope.payload, envelope.senderId);
      return;
    }

    if (isTopic(envelope, "whiteboard")) {
      this.handleWhiteboardEvent(envelope.payload);
    }
  }

  private handleControlEvent(event: ControlEvent, senderId: string): void {
    if (event.type === "mode.set") {
      this.mode = event.mode;
      this.whiteboardOwnerId =
        event.mode === "whiteboard" ? (event.ownerId ?? senderId) : null;
      return;
    }

    if (event.type === "room.state") {
      this.mode = event.state.mode;
      this.whiteboardOwnerId =
        event.state.mode === "whiteboard" ? (event.state.whiteboardOwnerId ?? null) : null;
      return;
    }

    if (event.type === "room.state.request" && this.isSharing) {
      this.send("control", {
        type: "room.state",
        state: {
          roomName: this.session.roomName,
          mode: this.mode,
          presenterId: this.session.participantId,
          whiteboardOwnerId: this.whiteboardOwnerId,
          annotationsLocked: false,
          updatedAt: Date.now()
        }
      });
    }
  }

  private handleWhiteboardEvent(event: WhiteboardEvent): void {
    if (event.type === "whiteboard.patch") {
      this.whiteboardElements = mergeWhiteboardElements(this.whiteboardElements, event.elements);
      this.whiteboardFiles = mergeWhiteboardFiles(this.whiteboardFiles, event.files ?? []);
      return;
    }

    if (event.type === "whiteboard.snapshot") {
      this.whiteboardElements = event.elements;
      this.whiteboardFiles = mergeWhiteboardFiles(this.whiteboardFiles, event.files ?? []);
      return;
    }

    if (event.type === "whiteboard.clear") {
      this.whiteboardElements = [];
      this.whiteboardFiles = [];
      return;
    }

    if (event.type === "whiteboard.request" && this.whiteboardElements.length > 0) {
      this.send("whiteboard", {
        type: "whiteboard.snapshot",
        elements: this.whiteboardElements,
        files: this.whiteboardFiles,
        backgroundColor: "#0b0d10"
      });
    }
  }
}
