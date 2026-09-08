import type { CanvasObject, PenObject, Point } from "./types";

export interface Bounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export function createId(prefix = "object"): string {
  const random = globalThis.crypto?.randomUUID?.() ?? Math.random().toString(36).slice(2);
  return `${prefix}-${random}`;
}

export function normalizeRect(start: Point, end: Point): Bounds {
  return {
    x: Math.min(start.x, end.x),
    y: Math.min(start.y, end.y),
    width: Math.abs(end.x - start.x),
    height: Math.abs(end.y - start.y),
  };
}

function distance(a: Point, b: Point): number {
  return Math.hypot(a.x - b.x, a.y - b.y);
}

export function erasePenAt(
  pen: PenObject,
  point: Point,
  radius: number,
): PenObject[] {
  const segments: number[][] = [];
  let current: number[] = [];

  for (let index = 0; index < pen.points.length; index += 2) {
    const candidate = { x: pen.points[index], y: pen.points[index + 1] };
    if (distance(candidate, point) <= radius) {
      if (current.length >= 4) segments.push(current);
      current = [];
    } else {
      current.push(candidate.x, candidate.y);
    }
  }

  if (current.length >= 4) segments.push(current);
  if (segments.length === 1 && segments[0].length === pen.points.length) {
    return [pen];
  }

  return segments.map((points, index) => ({
    ...pen,
    id: index === 0 ? pen.id : createId("pen"),
    points,
  }));
}

export function erasePensAt(
  objects: CanvasObject[],
  point: Point,
  radius: number,
): CanvasObject[] {
  return objects.reduce<CanvasObject[]>((result, object) => {
    if (object.type === "pen") result.push(...erasePenAt(object, point, radius));
    else result.push(object);
    return result;
  }, []);
}

export function objectBounds(object: CanvasObject): Bounds {
  const halfStroke = object.strokeWidth / 2;
  switch (object.type) {
    case "pen":
    case "arrow": {
      const xs = object.points.filter((_, index) => index % 2 === 0);
      const ys = object.points.filter((_, index) => index % 2 === 1);
      const minX = Math.min(...xs) - halfStroke;
      const minY = Math.min(...ys) - halfStroke;
      const maxX = Math.max(...xs) + halfStroke;
      const maxY = Math.max(...ys) + halfStroke;
      return { x: minX, y: minY, width: maxX - minX, height: maxY - minY };
    }
    case "rectangle":
      return {
        x: object.x - halfStroke,
        y: object.y - halfStroke,
        width: object.width + object.strokeWidth,
        height: object.height + object.strokeWidth,
      };
    case "ellipse":
      return {
        x: object.x - object.radiusX - halfStroke,
        y: object.y - object.radiusY - halfStroke,
        width: object.radiusX * 2 + object.strokeWidth,
        height: object.radiusY * 2 + object.strokeWidth,
      };
    case "text": {
      const lines = object.text.split("\n");
      return {
        x: object.x,
        y: object.y,
        width: object.width,
        height: Math.max(object.fontSize * 1.25, lines.length * object.fontSize * 1.25),
      };
    }
  }
}

export function contentBounds(objects: CanvasObject[]): Bounds | null {
  if (objects.length === 0) return null;
  const bounds = objects.map(objectBounds);
  const minX = Math.min(...bounds.map((item) => item.x));
  const minY = Math.min(...bounds.map((item) => item.y));
  const maxX = Math.max(...bounds.map((item) => item.x + item.width));
  const maxY = Math.max(...bounds.map((item) => item.y + item.height));
  return { x: minX, y: minY, width: maxX - minX, height: maxY - minY };
}
