export interface WhiteboardElement {
  id: string;
  version: number;
  versionNonce: number;
  isDeleted?: boolean;
  [key: string]: unknown;
}

export interface WhiteboardFile {
  id: string;
  url: string;
  mimeType: string;
}

export interface WhiteboardPatchEvent {
  type: "whiteboard.patch";
  elements: WhiteboardElement[];
  files?: WhiteboardFile[];
}

export interface WhiteboardSnapshotEvent {
  type: "whiteboard.snapshot";
  elements: WhiteboardElement[];
  files?: WhiteboardFile[];
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

export function mergeWhiteboardFiles(
  current: WhiteboardFile[],
  incoming: WhiteboardFile[]
): WhiteboardFile[] {
  const byId = new Map<string, WhiteboardFile>();
  for (const file of current) {
    byId.set(file.id, file);
  }
  for (const file of incoming) {
    if (!byId.has(file.id)) {
      byId.set(file.id, file);
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
