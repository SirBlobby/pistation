import type { CompletedStroke, NormalizedPoint } from "@pistation/shared-types";

import type { AnnotationState } from "./annotation-state.js";

export interface RenderTarget {
  context: CanvasRenderingContext2D;
  width: number;
  height: number;
}

export interface RenderOptions {
  /// Participant id to display name, used to label remote pointers.
  labels?: Map<string, string>;
}

export function renderAnnotations(
  target: RenderTarget,
  state: AnnotationState,
  options: RenderOptions = {}
): void {
  const { context, width, height } = target;
  context.clearRect(0, 0, width, height);

  for (const stroke of state.strokes) {
    drawStroke(target, stroke);
  }

  for (const pointer of state.pointers) {
    drawPointer(target, pointer.point, pointer.color, options.labels?.get(pointer.participantId));
  }
}

function drawStroke(target: RenderTarget, stroke: CompletedStroke): void {
  const { context, width, height } = target;
  if (stroke.points.length === 0) return;

  context.save();
  context.globalAlpha = stroke.style.opacity;
  context.strokeStyle = stroke.style.color;
  context.fillStyle = stroke.style.color;
  context.lineWidth = Math.max(1, stroke.style.width * width);
  context.lineCap = "round";
  context.lineJoin = "round";

  const first = toPixels(stroke.points[0], width, height);
  const last = toPixels(stroke.points[stroke.points.length - 1], width, height);

  switch (stroke.tool) {
    case "rectangle":
      context.strokeRect(first.x, first.y, last.x - first.x, last.y - first.y);
      break;

    case "ellipse": {
      const centerX = (first.x + last.x) / 2;
      const centerY = (first.y + last.y) / 2;
      context.beginPath();
      context.ellipse(
        centerX,
        centerY,
        Math.abs(last.x - first.x) / 2,
        Math.abs(last.y - first.y) / 2,
        0,
        0,
        Math.PI * 2
      );
      context.stroke();
      break;
    }

    case "arrow":
      drawArrow(context, first, last, context.lineWidth);
      break;

    case "laser":
      drawPointer(target, stroke.points[stroke.points.length - 1], stroke.style.color);
      break;

    default:
      drawFreehand(context, stroke.points, width, height);
      break;
  }

  context.restore();
}

function drawFreehand(
  context: CanvasRenderingContext2D,
  points: NormalizedPoint[],
  width: number,
  height: number
): void {
  context.beginPath();
  const start = toPixels(points[0], width, height);
  context.moveTo(start.x, start.y);

  for (let index = 1; index < points.length; index += 1) {
    const previous = toPixels(points[index - 1], width, height);
    const current = toPixels(points[index], width, height);
    const midpoint = { x: (previous.x + current.x) / 2, y: (previous.y + current.y) / 2 };
    context.quadraticCurveTo(previous.x, previous.y, midpoint.x, midpoint.y);
  }

  const final = toPixels(points[points.length - 1], width, height);
  context.lineTo(final.x, final.y);
  context.stroke();
}

function drawArrow(
  context: CanvasRenderingContext2D,
  from: { x: number; y: number },
  to: { x: number; y: number },
  lineWidth: number
): void {
  const deltaX = to.x - from.x;
  const deltaY = to.y - from.y;
  const length = Math.hypot(deltaX, deltaY);
  if (length < 1) return;

  // Unit vector along the arrow, and the perpendicular used for the head's width.
  const alongX = deltaX / length;
  const alongY = deltaY / length;
  const acrossX = -alongY;
  const acrossY = alongX;

  // The head grows with the line, but never takes more than part of a short arrow.
  const headLength = Math.min(length * 0.45, Math.max(lineWidth * 3.6, 10));
  const halfWidth = headLength * 0.42;

  const baseX = to.x - alongX * headLength;
  const baseY = to.y - alongY * headLength;

  // Stopping the shaft just inside the head keeps a thick round cap from bulging out
  // through the point of the arrow.
  context.beginPath();
  context.moveTo(from.x, from.y);
  context.lineTo(baseX + alongX * lineWidth * 0.5, baseY + alongY * lineWidth * 0.5);
  context.stroke();

  context.save();
  context.lineJoin = "miter";
  context.beginPath();
  context.moveTo(to.x, to.y);
  context.lineTo(baseX + acrossX * halfWidth, baseY + acrossY * halfWidth);
  context.lineTo(baseX - acrossX * halfWidth, baseY - acrossY * halfWidth);
  context.closePath();
  context.fill();
  context.restore();
}

function drawPointer(
  target: RenderTarget,
  point: NormalizedPoint,
  color: string,
  label?: string
): void {
  const { context, width, height } = target;
  const position = toPixels(point, width, height);
  const radius = Math.max(3, width * 0.0028);

  context.save();
  context.globalAlpha = 0.22;
  context.fillStyle = color;
  context.beginPath();
  context.arc(position.x, position.y, radius * 2.1, 0, Math.PI * 2);
  context.fill();

  context.globalAlpha = 1;
  context.beginPath();
  context.arc(position.x, position.y, radius, 0, Math.PI * 2);
  context.fill();

  if (label) {
    drawPointerLabel(context, position, radius, color, label, width);
  }

  context.restore();
}

function drawPointerLabel(
  context: CanvasRenderingContext2D,
  position: { x: number; y: number },
  radius: number,
  color: string,
  label: string,
  width: number
): void {
  const fontSize = Math.max(10, width * 0.0072);
  context.font = `600 ${fontSize}px Inter, system-ui, sans-serif`;
  context.textBaseline = "middle";

  const paddingX = fontSize * 0.5;
  const paddingY = fontSize * 0.3;
  const textWidth = context.measureText(label).width;
  const boxWidth = textWidth + paddingX * 2;
  const boxHeight = fontSize + paddingY * 2;

  const boxX = position.x + radius * 2.6;
  const boxY = position.y - boxHeight / 2;

  context.globalAlpha = 0.92;
  context.fillStyle = color;
  context.fillRect(boxX, boxY, boxWidth, boxHeight);

  context.globalAlpha = 1;
  context.fillStyle = "#0b0d10";
  context.fillText(label, boxX + paddingX, position.y);
}

function toPixels(point: NormalizedPoint, width: number, height: number) {
  return { x: point.x * width, y: point.y * height };
}
