import { describe, expect, it } from "vitest";
import { contentBounds, erasePenAt, normalizeRect } from "./geometry";
import type { PenObject } from "./types";

const pen: PenObject = {
  id: "pen-1",
  type: "pen",
  stroke: "#000000",
  strokeWidth: 4,
  points: [0, 0, 10, 0, 20, 0, 30, 0, 40, 0],
};

describe("geometry", () => {
  it("normalizes shapes drawn in any direction", () => {
    expect(normalizeRect({ x: 20, y: 30 }, { x: 5, y: 10 })).toEqual({
      x: 5,
      y: 10,
      width: 15,
      height: 20,
    });
  });

  it("splits a pen stroke around a partial erase", () => {
    const pieces = erasePenAt(pen, { x: 20, y: 0 }, 2);
    expect(pieces).toHaveLength(2);
    expect(pieces[0].points).toEqual([0, 0, 10, 0]);
    expect(pieces[1].points).toEqual([30, 0, 40, 0]);
  });

  it("calculates a content-only send boundary", () => {
    expect(contentBounds([pen])).toEqual({ x: -2, y: -2, width: 44, height: 4 });
  });
});

