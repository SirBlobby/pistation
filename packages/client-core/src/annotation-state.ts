import type {
  AnnotationEvent,
  CompletedStroke,
  NormalizedPoint,
  StrokeStyle
} from "@pistation/shared-types";
import { isPersistentTool } from "@pistation/shared-types";

export interface LiveStroke extends CompletedStroke {
  isOpen: boolean;
}

export interface RemotePointer {
  participantId: string;
  point: NormalizedPoint;
  color: string;
  updatedAt: number;
}

export interface AnnotationState {
  strokes: LiveStroke[];
  pointers: RemotePointer[];
}

export const POINTER_TIMEOUT_MS = 4000;

export function createAnnotationState(): AnnotationState {
  return { strokes: [], pointers: [] };
}

export function applyAnnotationEvent(
  state: AnnotationState,
  event: AnnotationEvent,
  authorId: string
): AnnotationState {
  switch (event.type) {
    case "stroke.start":
      return {
        ...state,
        strokes: [
          ...state.strokes.filter((stroke) => stroke.strokeId !== event.strokeId),
          {
            strokeId: event.strokeId,
            authorId,
            tool: event.tool,
            style: event.style,
            points: [event.point],
            isOpen: true
          }
        ]
      };

    case "stroke.append":
      return {
        ...state,
        strokes: state.strokes.map((stroke) =>
          stroke.strokeId === event.strokeId
            ? { ...stroke, points: [...stroke.points, ...event.points] }
            : stroke
        )
      };

    case "stroke.end":
      return {
        ...state,
        strokes: state.strokes
          .map((stroke) =>
            stroke.strokeId === event.strokeId ? { ...stroke, isOpen: false } : stroke
          )
          .filter((stroke) => isPersistentTool(stroke.tool))
      };

    case "stroke.erase":
      return {
        ...state,
        strokes: state.strokes.filter((stroke) => !event.strokeIds.includes(stroke.strokeId))
      };

    case "canvas.clear":
      return { ...state, strokes: [] };

    case "pointer.move": {
      const others = state.pointers.filter((pointer) => pointer.participantId !== authorId);
      if (!event.point) return { ...state, pointers: others };
      return {
        ...state,
        pointers: [
          ...others,
          {
            participantId: authorId,
            point: event.point,
            color: event.color,
            updatedAt: Date.now()
          }
        ]
      };
    }

    default:
      return state;
  }
}

export function prunePointers(state: AnnotationState, now: number): AnnotationState {
  const fresh = state.pointers.filter((pointer) => now - pointer.updatedAt < POINTER_TIMEOUT_MS);
  return fresh.length === state.pointers.length ? state : { ...state, pointers: fresh };
}

export function eraseAtPoint(
  state: AnnotationState,
  point: NormalizedPoint,
  radius: number
): string[] {
  const hits: string[] = [];
  for (const stroke of state.strokes) {
    if (stroke.points.some((candidate) => distance(candidate, point) <= radius)) {
      hits.push(stroke.strokeId);
    }
  }
  return hits;
}

/// Authors with a stroke still in progress, so the UI can say who is drawing right now.
export function activeAuthors(state: AnnotationState): string[] {
  const authors = new Set<string>();
  for (const stroke of state.strokes) {
    if (stroke.isOpen) authors.add(stroke.authorId);
  }
  return [...authors];
}

export function makeStrokeId(): string {
  return `s-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

export function withColor(style: StrokeStyle, color: string): StrokeStyle {
  return { ...style, color };
}

function distance(a: NormalizedPoint, b: NormalizedPoint): number {
  return Math.hypot(a.x - b.x, a.y - b.y);
}
