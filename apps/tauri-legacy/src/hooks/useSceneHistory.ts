import { useCallback, useMemo, useState } from "react";
import {
  commitScene,
  createHistory,
  currentScene,
  redoScene,
  resetHistory,
  undoScene,
} from "../domain/history";
import type { CanvasObject } from "../domain/types";

export function useSceneHistory() {
  const [history, setHistory] = useState(() => createHistory());
  const objects = useMemo(() => currentScene(history), [history]);

  const commit = useCallback((next: CanvasObject[]) => {
    setHistory((current) => commitScene(current, next));
  }, []);

  const undo = useCallback(() => setHistory(undoScene), []);
  const redo = useCallback(() => setHistory(redoScene), []);
  const reset = useCallback(
    (next: CanvasObject[] = []) => setHistory(resetHistory(next)),
    [],
  );

  return {
    objects,
    commit,
    undo,
    redo,
    reset,
    canUndo: history.index > 0,
    canRedo: history.index < history.snapshots.length - 1,
  };
}

