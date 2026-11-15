import { describe, expect, it } from "vitest";
// The dictionary the core grows from: the one list both sides read (world R8).
import items from "../../crates/pec-core/src/world/items.json";
import subjects from "../../crates/pec-core/src/world/subjects.json";
import { t } from "../strings";
import { hasDrawing } from "./drawings";

describe("world items", () => {
  // world FR-009, FR-011, journey FR-017: every item ships with its drawing and its name.
  it("has a drawing and a name for every item the core can draw", () => {
    expect(items.length).toBeGreaterThanOrEqual(150);
    for (const { id } of items) {
      expect(hasDrawing(id), `${id} has no drawing`).toBe(true);
      expect(t.world.name(id), `${id} has no name`).not.toBe(id);
    }
  });

  // journey FR-034: subject names come from the string table.
  it("names every subject", () => {
    for (const { id } of subjects) expect(t.world.subject(id), `${id} has no name`).not.toBe(id);
  });
});
