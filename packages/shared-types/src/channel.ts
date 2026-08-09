import type { AnnotationEvent } from "./annotation.js";
import type { ControlEvent } from "./control.js";
import type { WhiteboardEvent } from "./whiteboard.js";

export const DATA_TOPICS = ["annotation", "control", "whiteboard"] as const;

export type DataTopic = (typeof DATA_TOPICS)[number];

export const PROTOCOL_VERSION = 1;

export interface TopicPayloadMap {
  annotation: AnnotationEvent;
  control: ControlEvent;
  whiteboard: WhiteboardEvent;
}

export interface DataEnvelope<T extends DataTopic = DataTopic> {
  version: number;
  topic: T;
  senderId: string;
  sentAt: number;
  payload: TopicPayloadMap[T];
}

export function encodeEnvelope<T extends DataTopic>(
  topic: T,
  senderId: string,
  payload: TopicPayloadMap[T]
): Uint8Array {
  const envelope: DataEnvelope<T> = {
    version: PROTOCOL_VERSION,
    topic,
    senderId,
    sentAt: Date.now(),
    payload
  };
  return new TextEncoder().encode(JSON.stringify(envelope));
}

export function decodeEnvelope(data: Uint8Array): DataEnvelope | null {
  return parseEnvelope(new TextDecoder().decode(data));
}

export function parseEnvelope(json: string): DataEnvelope | null {
  try {
    const parsed = JSON.parse(json) as DataEnvelope;
    if (parsed.version !== PROTOCOL_VERSION) return null;
    if (!DATA_TOPICS.includes(parsed.topic)) return null;
    return parsed;
  } catch {
    return null;
  }
}

export function isTopic<T extends DataTopic>(
  envelope: DataEnvelope,
  topic: T
): envelope is DataEnvelope<T> {
  return envelope.topic === topic;
}
