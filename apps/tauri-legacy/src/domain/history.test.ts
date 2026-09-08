import { describe, expect, it } from "vitest";
import { commitScene, createHistory, currentScene, redoScene, undoScene } from "./history";
import type { RectangleObject } from "./types";

const rectangle: RectangleObject = {
  id: "rect-1",
  type: "rectangle",
  x: 10,
  y: 10,
  width: 40,
  height: 30,
  stroke: "#000000",
  strokeWidth: 3,
};

describe("scene history", () => {
  it("supports undo and redo", () => {
    const committed = commitScene(createHistory(), [rectangle]);
    expect(currentScene(undoScene(committed))).toEqual([]);
    expect(currentScene(redoScene(undoScene(committed)))).toEqual([rectangle]);
  });

  it("drops the redo branch after a new commit", () => {
    const first = commitScene(createHistory(), [rectangle]);
    const rewound = undoScene(first);
    const replacement = commitScene(rewound, [{ ...rectangle, id: "rect-2" }]);
    expect(replacement.snapshots).toHaveLength(2);
    expect(currentScene(redoScene(replacement))[0].id).toBe("rect-2");
  });
});
