export const ANNOTATION_TOOLS = [
  "pen",
  "highlighter",
  "arrow",
  "rectangle",
  "ellipse",
  "laser"
] as const;

export type AnnotationTool = (typeof ANNOTATION_TOOLS)[number];

export interface NormalizedPoint {
  x: number;
  y: number;
}

export interface StrokeStyle {
  color: string;
  width: number;
  opacity: number;
}

export interface StrokeStartEvent {
  type: "stroke.start";
  strokeId: string;
  tool: AnnotationTool;
  style: StrokeStyle;
  point: NormalizedPoint;
}

export interface StrokeAppendEvent {
  type: "stroke.append";
  strokeId: string;
  points: NormalizedPoint[];
}

export interface StrokeEndEvent {
  type: "stroke.end";
  strokeId: string;
}

export interface StrokeEraseEvent {
  type: "stroke.erase";
  strokeIds: string[];
}

export interface CanvasClearEvent {
  type: "canvas.clear";
}

export interface PointerMoveEvent {
  type: "pointer.move";
  point: NormalizedPoint | null;
  color: string;
}

export type AnnotationEvent =
  | StrokeStartEvent
  | StrokeAppendEvent
  | StrokeEndEvent
  | StrokeEraseEvent
  | CanvasClearEvent
  | PointerMoveEvent;

export interface CompletedStroke {
  strokeId: string;
  authorId: string;
  tool: AnnotationTool;
  style: StrokeStyle;
  points: NormalizedPoint[];
}

export const DEFAULT_STROKE_STYLE: StrokeStyle = {
  color: "#ff2d55",
  width: 0.004,
  opacity: 1
};

export const HIGHLIGHTER_STYLE: StrokeStyle = {
  color: "#ffd60a",
  width: 0.018,
  opacity: 0.35
};

export function isPersistentTool(tool: AnnotationTool): boolean {
  return tool !== "laser";
}
