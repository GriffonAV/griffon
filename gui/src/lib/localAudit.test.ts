import { test } from "node:test";
import assert from "node:assert/strict";
import {
  auditStatusLabel,
  buildMarkdownReport,
  buildSarifReport,
  compareAuditFindings,
  filterActiveBaseline,
  auditFilterClassName,
  auditFindingCardStyle,
  auditFindingCardClassName,
} from "./localAudit.ts";

const report = {
  status: "partial",
  message: "Audit completed",
  profile: "standard",
  scope: "test scope",
  duration_ms: 42,
  critical_count: 1,
  high_count: 0,
  medium_count: 0,
  low_count: 0,
  info_count: 1,
  truncated: false,
  diagnostics_summary: "one check unavailable",
  diagnostics: ["one check unavailable"],
  checks: [],
  findings: [
    {
      id: "finding-a",
      severity: "Critical",
      category: "SSH",
      title: "Root login",
      evidence: "PermitRootLogin yes",
      recommendation: "Disable it",
    },
  ],
  inventory: [],
};

test("labels partial reports without calling them clean", () => {
  assert.equal(auditStatusLabel("partial"), "Partial — some checks unavailable");
  assert.equal(auditStatusLabel("clean"), "Clean — no actionable findings");
});

test("exports findings as markdown and SARIF", () => {
  assert.match(buildMarkdownReport(report), /Root login/);
  const sarif = buildSarifReport(report);
  assert.equal(sarif.runs[0].results[0].ruleId, "finding-a");
});

test("compares added and resolved findings by stable id", () => {
  const comparison = compareAuditFindings(
    report.findings,
    [{ ...report.findings[0], id: "finding-b" }],
  );
  assert.deepEqual(comparison.added.map((finding) => finding.id), ["finding-a"]);
  assert.deepEqual(comparison.resolved.map((finding) => finding.id), ["finding-b"]);
});

test("filters expired baseline entries", () => {
  const entries = filterActiveBaseline(
    [
      { findingId: "active", expiresAt: Date.now() + 60_000 },
      { findingId: "expired", expiresAt: Date.now() - 1 },
    ],
    Date.now(),
  );
  assert.deepEqual(entries.map((entry) => entry.findingId), ["active"]);
});

test("audit filter controls use theme-aware foreground and input tokens", () => {
  assert.match(auditFilterClassName, /text-foreground/);
  assert.match(auditFilterClassName, /bg-input/);
});

test("finding cards opt into deferred offscreen rendering", () => {
  assert.equal(auditFindingCardStyle.contentVisibility, "auto");
  assert.match(auditFindingCardStyle.containIntrinsicSize, /180px/);
});

test("finding cards use the lightweight visual style", () => {
  assert.match(auditFindingCardClassName, /shadow-none/);
  assert.doesNotMatch(auditFindingCardClassName, /shadow-sm/);
});
