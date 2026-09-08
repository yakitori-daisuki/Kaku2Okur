import type { CanvasObject } from "./types";

export interface SceneHistory {
  snapshots: CanvasObject[][];
  index: number;
}

export function createHistory(initial: CanvasObject[] = []): SceneHistory {
  return { snapshots: [initial], index: 0 };
}

export function currentScene(history: SceneHistory): CanvasObject[] {
  return history.snapshots[history.index];
}

export function commitScene(
  history: SceneHistory,
  objects: CanvasObject[],
): SceneHistory {
  const current = currentScene(history);
  if (JSON.stringify(current) === JSON.stringify(objects)) return history;

  return {
    snapshots: [...history.snapshots.slice(0, history.index + 1), objects],
    index: history.index + 1,
  };
}

export function undoScene(history: SceneHistory): SceneHistory {
  if (history.index === 0) return history;
  return { ...history, index: history.index - 1 };
}

export function redoScene(history: SceneHistory): SceneHistory {
  if (history.index >= history.snapshots.length - 1) return history;
  return { ...history, index: history.index + 1 };
}

export function resetHistory(objects: CanvasObject[] = []): SceneHistory {
  return createHistory(objects);
}

