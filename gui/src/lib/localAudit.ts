export type AuditFinding = {
  id: string;
  severity: string;
  category: string;
  title: string;
  evidence: string;
  recommendation: string;
};

export type AuditCheck = {
  id: string;
  label: string;
  status: string;
  duration_ms: number;
  findings: number;
  inventory: number;
  diagnostics: number;
};

export type AuditReport = {
  ok?: boolean;
  status: string;
  message: string;
  profile: string;
  scope: string;
  duration_ms: number;
  critical_count: number;
  high_count: number;
  medium_count: number;
  low_count: number;
  info_count: number;
  truncated: boolean;
  diagnostics_summary: string;
  diagnostics: string[];
  checks: AuditCheck[];
  findings: AuditFinding[];
  inventory: Array<{ category: string; item: string; details: string }>;
};

export type BaselineEntry = {
  findingId: string;
  expiresAt: number | null;
};

export const auditFilterClassName = filterSelectClassName;

export const auditFindingCardStyle = {
  contentVisibility: "auto",
  containIntrinsicSize: "auto 180px",
} as const;

export const auditFindingCardClassName =
  "rounded-lg border border-l-2 bg-card p-3 shadow-none";

export function auditStatusLabel(status: string): string {
  switch (status) {
    case "clean":
      return "Clean — no actionable findings";
    case "findings":
      return "Findings require review";
    case "partial":
      return "Partial — some checks unavailable";
    case "truncated":
      return "Truncated — report limit reached";
    case "failed":
      return "Failed — no trustworthy result available";
    case "cancelled":
      return "Cancelled — report is incomplete";
    default:
      return "Not run";
  }
}

export function buildMarkdownReport(report: AuditReport): string {
  const lines = [
    "# Griffon local security audit",
    "",
    `- Status: ${auditStatusLabel(report.status)}`,
    `- Profile: ${report.profile}`,
    `- Scope: ${report.scope}`,
    `- Duration: ${report.duration_ms} ms`,
    "",
    "## Summary",
    "",
    `- Critical: ${report.critical_count}`,
    `- High: ${report.high_count}`,
    `- Medium: ${report.medium_count}`,
    `- Low: ${report.low_count}`,
    `- Inventory: ${report.info_count}`,
    "",
    "## Findings",
    "",
  ];

  for (const finding of report.findings) {
    lines.push(
      `### [${finding.severity}] ${finding.title}`,
      `- ID: \`${finding.id}\``,
      `- Category: ${finding.category}`,
      `- Evidence: \`${finding.evidence}\``,
      `- Recommendation: ${finding.recommendation}`,
      "",
    );
  }

  if (report.diagnostics.length > 0) {
    lines.push("## Diagnostics", "", ...report.diagnostics.map((item) => `- ${item}`), "");
  }

  return lines.join("\n");
}

export function buildSarifReport(report: AuditReport) {
  return {
    version: "2.1.0",
    $schema: "https://json.schemastore.org/sarif-2.1.0.json",
    runs: [
      {
        tool: { driver: { name: "Griffon Local Security Audit", version: "0.1.0" } },
        results: report.findings.map((finding) => ({
          ruleId: finding.id,
          level: sarifLevel(finding.severity),
          message: { text: `${finding.title}: ${finding.recommendation}` },
          properties: {
            category: finding.category,
            evidence: finding.evidence,
          },
        })),
      },
    ],
  };
}

function sarifLevel(severity: string): "error" | "warning" | "note" {
  if (severity === "Critical" || severity === "High") return "error";
  if (severity === "Medium") return "warning";
  return "note";
}

export function compareAuditFindings(
  current: AuditFinding[],
  previous: AuditFinding[],
): { added: AuditFinding[]; resolved: AuditFinding[] } {
  const previousIds = new Set(previous.map((finding) => finding.id));
  const currentIds = new Set(current.map((finding) => finding.id));
  return {
    added: current.filter((finding) => !previousIds.has(finding.id)),
    resolved: previous.filter((finding) => !currentIds.has(finding.id)),
  };
}

export function filterActiveBaseline(
  entries: BaselineEntry[],
  now = Date.now(),
): BaselineEntry[] {
  return entries.filter((entry) => entry.expiresAt === null || entry.expiresAt > now);
}
import { filterSelectClassName } from "./filterStyles.ts";

