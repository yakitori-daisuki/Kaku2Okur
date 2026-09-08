export type Tool =
  | "select"
  | "pen"
  | "eraser"
  | "arrow"
  | "rectangle"
  | "ellipse"
  | "text";

export type BackgroundMode = "system" | "white" | "black" | "custom";
export type SendBackgroundMode = "match" | "white" | "black" | "custom";
export type InvocationGesture = "dual-modifier" | "conventional";

interface ObjectBase {
  id: string;
  stroke: string;
  strokeWidth: number;
}

export interface PenObject extends ObjectBase {
  type: "pen";
  points: number[];
}

export interface ArrowObject extends ObjectBase {
  type: "arrow";
  points: [number, number, number, number];
}

export interface RectangleObject extends ObjectBase {
  type: "rectangle";
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface EllipseObject extends ObjectBase {
  type: "ellipse";
  x: number;
  y: number;
  radiusX: number;
  radiusY: number;
}

export interface TextObject extends ObjectBase {
  type: "text";
  x: number;
  y: number;
  width: number;
  text: string;
  fontSize: number;
}

export type CanvasObject =
  | PenObject
  | ArrowObject
  | RectangleObject
  | EllipseObject
  | TextObject;

export interface AppSettings {
  backgroundMode: BackgroundMode;
  customBackground: string;
  sendBackgroundMode: SendBackgroundMode;
  customSendBackground: string;
  invocationGesture: InvocationGesture;
  trackpadDefault: boolean;
}

export interface DeliveryOutcome {
  status: "dispatched" | "clipboardFallback" | "preview";
  message?: string;
}

export interface Point {
  x: number;
  y: number;
}

export interface TrackpadTouch {
  x: number;
  y: number;
  phase: 0 | 1 | 2 | 3;
  touchCount: number;
}
