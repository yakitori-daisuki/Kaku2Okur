import { contentBounds } from "./geometry";
import type { CanvasObject } from "./types";

const PADDING = 32;
const PIXEL_RATIO = 2;

function drawArrowHead(
  context: CanvasRenderingContext2D,
  points: [number, number, number, number],
  strokeWidth: number,
) {
  const [x1, y1, x2, y2] = points;
  const angle = Math.atan2(y2 - y1, x2 - x1);
  const length = Math.max(12, strokeWidth * 4);
  context.moveTo(x2, y2);
  context.lineTo(
    x2 - length * Math.cos(angle - Math.PI / 7),
    y2 - length * Math.sin(angle - Math.PI / 7),
  );
  context.moveTo(x2, y2);
  context.lineTo(
    x2 - length * Math.cos(angle + Math.PI / 7),
    y2 - length * Math.sin(angle + Math.PI / 7),
  );
}

function drawObject(context: CanvasRenderingContext2D, object: CanvasObject) {
  context.strokeStyle = object.stroke;
  context.fillStyle = object.stroke;
  context.lineWidth = object.strokeWidth;
  context.lineCap = "round";
  context.lineJoin = "round";
  context.beginPath();

  switch (object.type) {
    case "pen":
      if (object.points.length < 4) return;
      context.moveTo(object.points[0], object.points[1]);
      for (let index = 2; index < object.points.length; index += 2) {
        context.lineTo(object.points[index], object.points[index + 1]);
      }
      context.stroke();
      break;
    case "arrow":
      context.moveTo(object.points[0], object.points[1]);
      context.lineTo(object.points[2], object.points[3]);
      drawArrowHead(context, object.points, object.strokeWidth);
      context.stroke();
      break;
    case "rectangle":
      context.strokeRect(object.x, object.y, object.width, object.height);
      break;
    case "ellipse":
      context.ellipse(
        object.x,
        object.y,
        object.radiusX,
        object.radiusY,
        0,
        0,
        Math.PI * 2,
      );
      context.stroke();
      break;
    case "text": {
      context.font = `500 ${object.fontSize}px -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif`;
      context.textBaseline = "top";
      object.text.split("\n").forEach((line, index) => {
        context.fillText(line, object.x, object.y + index * object.fontSize * 1.25);
      });
      break;
    }
  }
}

export function renderSceneToDataUrl(
  objects: CanvasObject[],
  background: string,
): string | null {
  const bounds = contentBounds(objects);
  if (!bounds) return null;

  const width = Math.max(1, Math.ceil(bounds.width + PADDING * 2));
  const height = Math.max(1, Math.ceil(bounds.height + PADDING * 2));
  const canvas = document.createElement("canvas");
  canvas.width = width * PIXEL_RATIO;
  canvas.height = height * PIXEL_RATIO;

  const context = canvas.getContext("2d");
  if (!context) throw new Error("Canvas rendering is unavailable");

  context.scale(PIXEL_RATIO, PIXEL_RATIO);
  context.fillStyle = background;
  context.fillRect(0, 0, width, height);
  context.translate(PADDING - bounds.x, PADDING - bounds.y);
  objects.forEach((object) => drawObject(context, object));
  return canvas.toDataURL("image/png");
}

