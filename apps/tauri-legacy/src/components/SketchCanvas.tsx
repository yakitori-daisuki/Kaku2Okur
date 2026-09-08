import Konva from "konva";
import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useMemo,
  useRef,
  useState,
} from "react";
import { Arrow, Ellipse, Layer, Line, Rect, Stage, Text, Transformer } from "react-konva";
import { createId, erasePensAt, normalizeRect } from "../domain/geometry";
import type {
  CanvasObject,
  PenObject,
  Point,
  TextObject,
  Tool,
  TrackpadTouch,
} from "../domain/types";

export interface SketchCanvasHandle {
  cancelActiveEdit: () => boolean;
  handleTrackpadTouch: (touch: TrackpadTouch) => void;
}

interface SketchCanvasProps {
  objects: CanvasObject[];
  tool: Tool;
  stroke: string;
  strokeWidth: number;
  background: string;
  darkCanvas: boolean;
  selectedId: string | null;
  onSelectionChange: (id: string | null) => void;
  onCommit: (objects: CanvasObject[]) => void;
  onSend: (objects: CanvasObject[]) => void;
}

interface TextDraft {
  id: string;
  x: number;
  y: number;
  width: number;
  value: string;
  editingId?: string;
}

const MIN_SHAPE_SIZE = 4;
const ERASER_RADIUS = 12;

function movedObject(object: CanvasObject, node: Konva.Node): CanvasObject {
  const x = node.x();
  const y = node.y();

  switch (object.type) {
    case "pen":
    case "arrow": {
      const moved = {
        ...object,
        points: object.points.map((value, index) => value + (index % 2 === 0 ? x : y)) as never,
      };
      node.position({ x: 0, y: 0 });
      return moved;
    }
    case "rectangle":
    case "ellipse":
    case "text":
      return { ...object, x, y };
  }
}

function transformedObject(object: CanvasObject, node: Konva.Node): CanvasObject {
  const scaleX = node.scaleX();
  const scaleY = node.scaleY();
  const x = node.x();
  const y = node.y();
  node.scale({ x: 1, y: 1 });

  switch (object.type) {
    case "pen":
    case "arrow": {
      const points = object.points.map((value, index) =>
        index % 2 === 0 ? value * scaleX + x : value * scaleY + y,
      );
      node.position({ x: 0, y: 0 });
      return { ...object, points: points as never };
    }
    case "rectangle":
      return {
        ...object,
        x,
        y,
        width: Math.max(MIN_SHAPE_SIZE, object.width * Math.abs(scaleX)),
        height: Math.max(MIN_SHAPE_SIZE, object.height * Math.abs(scaleY)),
      };
    case "ellipse":
      return {
        ...object,
        x,
        y,
        radiusX: Math.max(MIN_SHAPE_SIZE, object.radiusX * Math.abs(scaleX)),
        radiusY: Math.max(MIN_SHAPE_SIZE, object.radiusY * Math.abs(scaleY)),
      };
    case "text":
      return {
        ...object,
        x,
        y,
        width: Math.max(40, object.width * Math.abs(scaleX)),
        fontSize: Math.max(10, object.fontSize * Math.abs(scaleY)),
      };
  }
}

export const SketchCanvas = forwardRef<SketchCanvasHandle, SketchCanvasProps>(
  function SketchCanvas(
    {
      objects,
      tool,
      stroke,
      strokeWidth,
      background,
      darkCanvas,
      selectedId,
      onSelectionChange,
      onCommit,
      onSend,
    },
    ref,
  ) {
    const hostRef = useRef<HTMLDivElement>(null);
    const stageRef = useRef<Konva.Stage>(null);
    const transformerRef = useRef<Konva.Transformer>(null);
    const objectNodes = useRef(new Map<string, Konva.Node>());
    const drawingStart = useRef<Point | null>(null);
    const drawing = useRef(false);
    const trackpadDraft = useRef<PenObject | null>(null);
    const [size, setSize] = useState({ width: 800, height: 560 });
    const [draft, setDraft] = useState<CanvasObject | null>(null);
    const [eraserPreview, setEraserPreview] = useState<CanvasObject[] | null>(null);
    const [textDraft, setTextDraft] = useState<TextDraft | null>(null);
    const textAreaRef = useRef<HTMLTextAreaElement>(null);

    const visibleObjects = eraserPreview ?? objects;

    useEffect(() => {
      if (!hostRef.current) return;
      const observer = new ResizeObserver(([entry]) => {
        const { width, height } = entry.contentRect;
        setSize({ width: Math.max(1, width), height: Math.max(1, height) });
      });
      observer.observe(hostRef.current);
      return () => observer.disconnect();
    }, []);

    useEffect(() => {
      if (!selectedId || !transformerRef.current) {
        transformerRef.current?.nodes([]);
        return;
      }
      const node = objectNodes.current.get(selectedId);
      transformerRef.current.nodes(node ? [node] : []);
      transformerRef.current.getLayer()?.batchDraw();
    }, [selectedId, objects]);

    useEffect(() => {
      if (textDraft) requestAnimationFrame(() => textAreaRef.current?.focus());
    }, [textDraft?.id]);

    useEffect(() => {
      if (selectedId && !objects.some((object) => object.id === selectedId)) {
        onSelectionChange(null);
      }
    }, [objects, onSelectionChange, selectedId]);

    useImperativeHandle(ref, () => ({
      cancelActiveEdit() {
        if (textDraft || draft || eraserPreview) {
          setTextDraft(null);
          setDraft(null);
          setEraserPreview(null);
          drawing.current = false;
          drawingStart.current = null;
          return true;
        }
        return false;
      },
      handleTrackpadTouch(touch) {
        if (touch.touchCount !== 1) {
          if (touch.phase !== 2) {
            trackpadDraft.current = null;
            setDraft(null);
          }
          return;
        }

        const point = { x: touch.x * size.width, y: touch.y * size.height };
        if (touch.phase === 0) {
          const next: PenObject = {
            id: createId("trackpad-pen"),
            type: "pen",
            stroke,
            strokeWidth,
            points: [point.x, point.y],
          };
          trackpadDraft.current = next;
          setDraft(next);
          return;
        }

        if (touch.phase === 1 && trackpadDraft.current) {
          const next = {
            ...trackpadDraft.current,
            points: [...trackpadDraft.current.points, point.x, point.y],
          };
          trackpadDraft.current = next;
          setDraft(next);
          return;
        }

        if ((touch.phase === 2 || touch.phase === 3) && trackpadDraft.current) {
          const completed = trackpadDraft.current;
          trackpadDraft.current = null;
          setDraft(null);
          if (touch.phase === 2 && completed.points.length >= 4) {
            onCommit([...objects, completed]);
          }
        }
      },
    }));

    const pointer = (): Point | null => {
      const value = stageRef.current?.getPointerPosition();
      return value ? { x: value.x, y: value.y } : null;
    };

    const eraseAt = (point: Point) => {
      const source = eraserPreview ?? objects;
      let next = erasePensAt(source, point, ERASER_RADIUS);
      const hitId = stageRef.current?.getIntersection(point)?.id();
      if (hitId) {
        const hit = next.find((object) => object.id === hitId);
        if (hit && hit.type !== "pen") next = next.filter((object) => object.id !== hitId);
      }
      setEraserPreview(next);
    };

    const handlePointerDown = () => {
      const point = pointer();
      if (!point || textDraft) return;

      if (tool === "select") {
        const target = stageRef.current?.getIntersection(point);
        if (!target?.id()) onSelectionChange(null);
        return;
      }

      if (tool === "text") {
        setTextDraft({ id: createId("text-draft"), x: point.x, y: point.y, width: 220, value: "" });
        return;
      }

      drawing.current = true;
      drawingStart.current = point;

      if (tool === "eraser") {
        setEraserPreview(objects);
        eraseAt(point);
        return;
      }

      const base = { id: createId(tool), stroke, strokeWidth };
      if (tool === "pen") {
        setDraft({ ...base, type: "pen", points: [point.x, point.y] });
      } else if (tool === "arrow") {
        setDraft({ ...base, type: "arrow", points: [point.x, point.y, point.x, point.y] });
      } else if (tool === "rectangle") {
        setDraft({ ...base, type: "rectangle", x: point.x, y: point.y, width: 0, height: 0 });
      } else if (tool === "ellipse") {
        setDraft({ ...base, type: "ellipse", x: point.x, y: point.y, radiusX: 0, radiusY: 0 });
      }
    };

    const handlePointerMove = () => {
      if (!drawing.current) return;
      const point = pointer();
      const start = drawingStart.current;
      if (!point || !start) return;

      if (tool === "eraser") {
        eraseAt(point);
        return;
      }

      setDraft((current) => {
        if (!current) return current;
        if (current.type === "pen") {
          return { ...current, points: [...current.points, point.x, point.y] };
        }
        if (current.type === "arrow") {
          return { ...current, points: [start.x, start.y, point.x, point.y] };
        }
        const rect = normalizeRect(start, point);
        if (current.type === "rectangle") return { ...current, ...rect };
        if (current.type === "ellipse") {
          return {
            ...current,
            x: rect.x + rect.width / 2,
            y: rect.y + rect.height / 2,
            radiusX: rect.width / 2,
            radiusY: rect.height / 2,
          };
        }
        return current;
      });
    };

    const handlePointerUp = () => {
      if (!drawing.current) return;
      drawing.current = false;
      drawingStart.current = null;

      if (tool === "eraser") {
        if (eraserPreview) onCommit(eraserPreview);
        setEraserPreview(null);
        return;
      }

      if (draft) {
        const valid =
          draft.type === "pen"
            ? draft.points.length >= 4
            : draft.type === "arrow"
              ? Math.hypot(draft.points[2] - draft.points[0], draft.points[3] - draft.points[1]) >= 4
              : draft.type === "rectangle"
                ? draft.width >= MIN_SHAPE_SIZE && draft.height >= MIN_SHAPE_SIZE
                : draft.type === "ellipse"
                  ? draft.radiusX >= MIN_SHAPE_SIZE && draft.radiusY >= MIN_SHAPE_SIZE
                  : false;
        if (valid) onCommit([...objects, draft]);
      }
      setDraft(null);
    };

    const commitText = (send: boolean) => {
      if (!textDraft) return;
      const value = textDraft.value.trimEnd();
      if (!value) {
        setTextDraft(null);
        return;
      }

      const textObject: TextObject = {
        id: textDraft.editingId ?? createId("text"),
        type: "text",
        x: textDraft.x,
        y: textDraft.y,
        width: textDraft.width,
        text: value,
        fontSize: 22,
        stroke,
        strokeWidth,
      };
      const next = textDraft.editingId
        ? objects.map((object) => (object.id === textDraft.editingId ? textObject : object))
        : [...objects, textObject];
      setTextDraft(null);
      onCommit(next);
      if (send) onSend(next);
    };

    const editText = (object: TextObject) => {
      setTextDraft({
        id: createId("text-draft"),
        x: object.x,
        y: object.y,
        width: object.width,
        value: object.text,
        editingId: object.id,
      });
    };

    const updateObject = (id: string, next: CanvasObject) => {
      onCommit(objects.map((object) => (object.id === id ? next : object)));
    };

    const renderObject = (object: CanvasObject) => {
      const common = {
        id: object.id,
        ref: (node: Konva.Node | null) => {
          if (node) objectNodes.current.set(object.id, node);
          else objectNodes.current.delete(object.id);
        },
        stroke: object.stroke,
        strokeWidth: object.strokeWidth,
        lineCap: "round" as const,
        lineJoin: "round" as const,
        draggable: tool === "select",
        onPointerDown: () => {
          if (tool === "select") onSelectionChange(object.id);
        },
        onDragStart: () => onSelectionChange(object.id),
        onDragEnd: (event: Konva.KonvaEventObject<DragEvent>) =>
          updateObject(object.id, movedObject(object, event.target)),
        onTransformEnd: (event: Konva.KonvaEventObject<Event>) =>
          updateObject(object.id, transformedObject(object, event.target)),
      };

      switch (object.type) {
        case "pen":
          return (
            <Line
              key={object.id}
              {...common}
              points={object.points}
              tension={0.35}
              hitStrokeWidth={18}
            />
          );
        case "arrow":
          return (
            <Arrow
              key={object.id}
              {...common}
              points={object.points}
              pointerLength={Math.max(10, object.strokeWidth * 4)}
              pointerWidth={Math.max(9, object.strokeWidth * 3.5)}
              hitStrokeWidth={18}
            />
          );
        case "rectangle":
          return (
            <Rect
              key={object.id}
              {...common}
              x={object.x}
              y={object.y}
              width={object.width}
              height={object.height}
            />
          );
        case "ellipse":
          return (
            <Ellipse
              key={object.id}
              {...common}
              x={object.x}
              y={object.y}
              radiusX={object.radiusX}
              radiusY={object.radiusY}
            />
          );
        case "text":
          return (
            <Text
              key={object.id}
              {...common}
              x={object.x}
              y={object.y}
              width={object.width}
              text={object.text}
              fill={object.stroke}
              strokeEnabled={false}
              fontSize={object.fontSize}
              fontFamily='-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
              fontStyle="500"
              lineHeight={1.25}
              onDblClick={() => editText(object)}
              onDblTap={() => editText(object)}
            />
          );
      }
    };

    const cursorClass = useMemo(() => `tool-${tool}`, [tool]);

    return (
      <div ref={hostRef} className={`canvas-host ${cursorClass}`} style={{ background }}>
        <Stage
          ref={stageRef}
          width={size.width}
          height={size.height}
          onPointerDown={handlePointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerUp}
          onPointerLeave={handlePointerUp}
        >
          <Layer>
            <Rect
              width={size.width}
              height={size.height}
              fill={background}
              listening={false}
            />
            {visibleObjects.map(renderObject)}
            {draft && renderObject(draft)}
          </Layer>
          <Layer>
            <Transformer
              ref={transformerRef}
              rotateEnabled={false}
              flipEnabled={false}
              borderStroke="#1769ff"
              anchorStroke="#1769ff"
              anchorFill={darkCanvas ? "#111315" : "#ffffff"}
              anchorSize={8}
              borderStrokeWidth={1.5}
              padding={4}
              ignoreStroke
            />
          </Layer>
        </Stage>

        {textDraft && (
          <textarea
            ref={textAreaRef}
            className="canvas-text-editor"
            style={{
              left: textDraft.x,
              top: textDraft.y,
              width: textDraft.width,
              color: stroke,
              caretColor: stroke,
            }}
            value={textDraft.value}
            aria-label="Canvas text"
            onChange={(event) =>
              setTextDraft((current) =>
                current ? { ...current, value: event.target.value } : current,
              )
            }
            onBlur={() => commitText(false)}
            onKeyDown={(event) => {
              if (event.key === "Enter" && !event.shiftKey) {
                event.preventDefault();
                commitText(true);
              } else if (event.key === "Escape") {
                event.preventDefault();
                event.stopPropagation();
                setTextDraft(null);
              }
            }}
          />
        )}
      </div>
    );
  },
);
