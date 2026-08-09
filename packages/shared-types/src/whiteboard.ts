export interface WhiteboardElement {
  id: string;
  version: number;
  versionNonce: number;
  isDeleted?: boolean;
  [key: string]: unknown;
}

export interface WhiteboardPatchEvent {
  type: "whiteboard.patch";
  elements: WhiteboardElement[];
}

export interface WhiteboardSnapshotEvent {
  type: "whiteboard.snapshot";
  elements: WhiteboardElement[];
  backgroundColor: string;
}

export interface WhiteboardRequestEvent {
  type: "whiteboard.request";
}

export interface WhiteboardClearEvent {
  type: "whiteboard.clear";
}

export type WhiteboardEvent =
  | WhiteboardPatchEvent
  | WhiteboardSnapshotEvent
  | WhiteboardRequestEvent
  | WhiteboardClearEvent;

export function mergeWhiteboardElements(
  current: WhiteboardElement[],
  incoming: WhiteboardElement[]
): WhiteboardElement[] {
  const byId = new Map<string, WhiteboardElement>();
  for (const element of current) {
    byId.set(element.id, element);
  }
  for (const element of incoming) {
    const existing = byId.get(element.id);
    if (!existing || isNewerElement(element, existing)) {
      byId.set(element.id, element);
    }
  }
  return [...byId.values()];
}

function isNewerElement(candidate: WhiteboardElement, existing: WhiteboardElement): boolean {
  if (candidate.version !== existing.version) {
    return candidate.version > existing.version;
  }
  return candidate.versionNonce < existing.versionNonce;
}
