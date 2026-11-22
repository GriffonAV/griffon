import { test } from "node:test";
import assert from "node:assert/strict";
import {
  buildCleanerSelectionValue,
  toggleCleanerSelectionValue,
} from "./cleanerSelection.ts";

test("selecting a Cleaner candidate preserves the active log filter", () => {
  const selectedPaths: string[] = [];
  const fileTypes = ["log"];

  const value = toggleCleanerSelectionValue(
    "/tmp/griffon-atp-v033",
    selectedPaths,
    true,
    fileTypes,
  );

  assert.deepEqual(value, {
    paths: ["/tmp/griffon-atp-v033"],
    dry_run: true,
    file_types: ["log"],
  });
  assert.deepEqual(selectedPaths, []);
  assert.deepEqual(fileTypes, ["log"]);
});

test("deselecting a Cleaner candidate preserves the active filters", () => {
  const value = toggleCleanerSelectionValue(
    "/tmp/griffon-atp-v033",
    ["/tmp/griffon-atp-v033", "/tmp/griffon-cleaner-test-logs"],
    true,
    ["log", "json"],
  );

  assert.deepEqual(value, {
    paths: ["/tmp/griffon-cleaner-test-logs"],
    dry_run: true,
    file_types: ["log", "json"],
  });
});

test("Cleaner selection payload preserves real deletion mode", () => {
  assert.deepEqual(
    buildCleanerSelectionValue(["/tmp/griffon-atp-v033"], false, ["log"]),
    {
      paths: ["/tmp/griffon-atp-v033"],
      dry_run: false,
      file_types: ["log"],
    },
  );
});

test("Cleaner selection payload preserves multiple filters", () => {
  assert.deepEqual(
    buildCleanerSelectionValue(
      ["/tmp/griffon-atp-v033"],
      true,
      ["log", "json"],
    ),
    {
      paths: ["/tmp/griffon-atp-v033"],
      dry_run: true,
      file_types: ["log", "json"],
    },
  );
});

test("Cleaner selection payload preserves an intentionally empty filter", () => {
  assert.deepEqual(
    buildCleanerSelectionValue(["/tmp/griffon-atp-v033"], true, []),
    {
      paths: ["/tmp/griffon-atp-v033"],
      dry_run: true,
      file_types: [],
    },
  );
});
