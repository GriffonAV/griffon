import { test } from "node:test";
import assert from "node:assert/strict";
import { filterSelectClassName } from "./filterStyles.ts";

test("table filter controls use theme-aware colors", () => {
  assert.match(filterSelectClassName, /text-foreground/);
  assert.match(filterSelectClassName, /bg-input/);
});
