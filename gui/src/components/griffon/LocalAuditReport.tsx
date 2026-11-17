import { memo, useCallback, useMemo, useState } from "react";
import { CheckCircle2, Clipboard, Download, Filter, Play, Save, ShieldAlert } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { filterSelectClassName } from "@/lib/filterStyles";
import { resolveFromPath } from "@/lib/utils";
import {
  auditStatusLabel,
  auditFindingCardClassName,
  auditFindingCardStyle,
  buildMarkdownReport,
  buildSarifReport,
  compareAuditFindings,
  filterActiveBaseline,
  type AuditReport,
  type BaselineEntry,
} from "@/lib/localAudit";

type Props = {
  element: { id: string; from?: string; plugin_uuid?: string };
  store: Record<string, any>;
  onAction?: (action: string, event?: any) => Promise<void> | void;
};

const BASELINE_PREFIX = "griffon.local-audit.baseline.";
const SNAPSHOT_PREFIX = "griffon.local-audit.snapshot.";

function tone(status: string) {
  if (status === "clean") return "border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300";
  if (status === "findings" || status === "truncated") return "border-amber-500/30 bg-amber-500/10 text-amber-800 dark:text-amber-300";
  if (status === "partial" || status === "failed") return "border-red-500/30 bg-red-500/10 text-red-700 dark:text-red-300";
  return "border-border bg-muted text-muted-foreground";
}

function reportFromStore(store: Record<string, any>, path?: string): AuditReport | null {
  const value = path ? resolveFromPath(path, { store }) : resolveFromPath("store.data.audit_result", { store });
  if (!value || typeof value !== "object") return null;
  return value as AuditReport;
}

function storageKey(prefix: string, element: Props["element"]) {
  return `${prefix}${element.plugin_uuid ?? "local-audit"}`;
}

function saveJson(filename: string, value: unknown, mime = "application/json") {
  const content = typeof value === "string" ? value : JSON.stringify(value, null, 2);
  const blob = new Blob([content], { type: mime });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  URL.revokeObjectURL(url);
}

export default function LocalAuditReport({
  element,
  store,
  onAction,
}: Props) {
  const report = reportFromStore(store, element.from);
  const pluginKey = storageKey("", element);
  const [query, setQuery] = useState("");
  const [severity, setSeverity] = useState("");
  const [category, setCategory] = useState("");
  const [baseline, setBaseline] = useState<BaselineEntry[]>(() => {
    try {
      return filterActiveBaseline(JSON.parse(localStorage.getItem(`${BASELINE_PREFIX}${pluginKey}`) ?? "[]"));
    } catch {
      return [];
    }
  });
  const [snapshot, setSnapshot] = useState<AuditReport | null>(() => {
    try {
      const raw = localStorage.getItem(`${SNAPSHOT_PREFIX}${pluginKey}`);
      return raw ? JSON.parse(raw) as AuditReport : null;
    } catch {
      return null;
    }
  });

  const currentStatus = report?.status && report.status !== "not_run" ? report.status : "not_run";
  const filteredFindings = useMemo(() => {
    const findings = report?.findings ?? [];
    const needle = query.trim().toLocaleLowerCase();
    return findings.filter((finding) => {
      if (severity && finding.severity !== severity) return false;
      if (category && finding.category !== category) return false;
      if (!needle) return true;
      return [finding.title, finding.category, finding.evidence, finding.recommendation]
        .some((value) => value.toLocaleLowerCase().includes(needle));
    });
  }, [category, query, report?.findings, severity]);

  const categories = Array.from(new Set((report?.findings ?? []).map((finding) => finding.category))).sort();
  const severities = ["Critical", "High", "Medium", "Low"];
  const comparison = snapshot && report ? compareAuditFindings(report.findings, snapshot.findings) : null;

  const ignoreFinding = useCallback((findingId: string) => {
    setBaseline((current) => {
      const active = filterActiveBaseline(current);
      const next = active.some((entry) => entry.findingId === findingId)
        ? active.filter((entry) => entry.findingId !== findingId)
        : [...active, {
          findingId,
          expiresAt: Date.now() + 7 * 24 * 60 * 60 * 1000,
        }];
      localStorage.setItem(`${BASELINE_PREFIX}${pluginKey}`, JSON.stringify(next));
      return next;
    });
  }, [pluginKey]);

  function saveSnapshot() {
    if (!report) return;
    localStorage.setItem(`${SNAPSHOT_PREFIX}${pluginKey}`, JSON.stringify(report));
    setSnapshot(report);
  }

  if (!report || currentStatus === "not_run") {
    return (
      <section id={element.id} className="rounded-2xl border bg-card p-5 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <h2 className="text-xl font-semibold">Audit readiness</h2>
            <p className="mt-1 text-sm text-muted-foreground">No audit has run yet. Results will be read-only and scoped to the runner environment.</p>
          </div>
          <Button onClick={() => onAction?.("local_audit.run")}><Play /> Run audit</Button>
        </div>
      </section>
    );
  }

  const suppressed = new Set(filterActiveBaseline(baseline).map((entry) => entry.findingId));

  return (
    <section id={element.id} className="space-y-4">
      <div className={`rounded-2xl border p-5 shadow-sm ${tone(currentStatus)}`}>
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div className="flex items-start gap-3">
            {currentStatus === "clean" ? <CheckCircle2 className="mt-1 size-6" /> : <ShieldAlert className="mt-1 size-6" />}
            <div>
              <h2 className="text-xl font-semibold">{auditStatusLabel(currentStatus)}</h2>
              <p className="mt-1 text-sm opacity-85">{report.message}</p>
              <p className="mt-1 text-xs opacity-75">Profile {report.profile} · {report.duration_ms} ms · {report.scope}</p>
            </div>
          </div>
          <div className="flex flex-wrap gap-2">
            <Button onClick={() => onAction?.("local_audit.run")}><Play /> Run again</Button>
            <Button variant="outline" onClick={saveSnapshot}><Save /> Save snapshot</Button>
          </div>
        </div>
        {report.truncated ? <p className="mt-3 rounded-lg border border-amber-500/30 bg-amber-500/10 p-3 text-sm">The report reached its configured limit. Increase the limit or use the deep profile before treating the inventory as complete.</p> : null}
      </div>

      <div className="grid grid-cols-2 gap-3 md:grid-cols-5">
        {[["Critical", report.critical_count], ["High", report.high_count], ["Medium", report.medium_count], ["Low", report.low_count], ["Inventory", report.info_count]].map(([label, value]) => (
          <div key={String(label)} className="rounded-xl border bg-card p-4 shadow-sm">
            <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">{label}</p>
            <p className="mt-2 text-3xl font-bold tabular-nums">{value}</p>
          </div>
        ))}
      </div>

      <div className="flex flex-wrap gap-2 rounded-xl border bg-card p-3">
        <Button size="sm" variant="outline" onClick={() => saveJson("griffon-audit.json", report)}><Download /> JSON</Button>
        <Button size="sm" variant="outline" onClick={() => saveJson("griffon-audit.md", buildMarkdownReport(report), "text/markdown")}><Download /> Markdown</Button>
        <Button size="sm" variant="outline" onClick={() => saveJson("griffon-audit.sarif", buildSarifReport(report))}><Download /> SARIF</Button>
        {comparison ? <span className="ml-auto self-center text-xs text-muted-foreground">{comparison.added.length} added · {comparison.resolved.length} resolved since snapshot</span> : null}
      </div>

      {(report.checks.length > 0 || report.diagnostics.length > 0) ? (
        <details className="rounded-xl border bg-card p-4">
          <summary className="cursor-pointer font-medium">Coverage and diagnostics</summary>
          <div className="mt-4 space-y-2">
            {report.checks.map((check) => <div key={check.id} className="flex flex-wrap items-center justify-between gap-2 rounded-lg bg-muted/40 px-3 py-2 text-sm"><span>{check.label}</span><span className={check.status === "complete" ? "text-emerald-600" : "text-amber-700"}>{check.status} · {check.duration_ms} ms</span></div>)}
            {report.diagnostics.map((diagnostic) => <p key={diagnostic} className="text-sm text-red-700 dark:text-red-300">{diagnostic}</p>)}
          </div>
        </details>
      ) : null}

      <div className="space-y-3">
        <div className="flex flex-col gap-2 rounded-xl border bg-card p-3 lg:flex-row">
          <Input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search findings…" aria-label="Search findings" />
          <Select value={severity || "__all__"} onValueChange={(value) => setSeverity(value === "__all__" ? "" : value)}>
            <SelectTrigger className={filterSelectClassName} aria-label="Filter by severity"><SelectValue placeholder="All severities" /></SelectTrigger>
            <SelectContent>
              <SelectItem value="__all__">All severities</SelectItem>
              {severities.map((value) => <SelectItem key={value} value={value}>{value}</SelectItem>)}
            </SelectContent>
          </Select>
          <Select value={category || "__all__"} onValueChange={(value) => setCategory(value === "__all__" ? "" : value)}>
            <SelectTrigger className={filterSelectClassName} aria-label="Filter by category"><SelectValue placeholder="All categories" /></SelectTrigger>
            <SelectContent>
              <SelectItem value="__all__">All categories</SelectItem>
              {categories.map((value) => <SelectItem key={value} value={value}>{value}</SelectItem>)}
            </SelectContent>
          </Select>
          <span className="flex items-center gap-1 text-xs text-muted-foreground"><Filter className="size-3" /> {filteredFindings.length} shown</span>
        </div>
        {filteredFindings.map((finding) => (
          <FindingCard
            key={finding.id}
            finding={finding}
            suppressed={suppressed.has(finding.id)}
            onIgnore={ignoreFinding}
          />
        ))}
        {filteredFindings.length === 0 ? <div className="rounded-xl border border-dashed p-10 text-center text-sm text-muted-foreground">No findings match the current filters.</div> : null}
      </div>
    </section>
  );
}

const FindingCard = memo(function FindingCard({
  finding,
  suppressed,
  onIgnore,
}: {
  finding: import("@/lib/localAudit").AuditFinding;
  suppressed: boolean;
  onIgnore: (findingId: string) => void;
}) {
  const [copied, setCopied] = useState<string | null>(null);

  async function copy(kind: string, value: string) {
    await navigator.clipboard?.writeText(value);
    setCopied(kind);
    window.setTimeout(() => setCopied(null), 1200);
  }

  return (
    <article
      style={auditFindingCardStyle}
      className={`${auditFindingCardClassName} ${finding.severity === "Critical" || finding.severity === "High" ? "border-l-red-500" : "border-l-amber-500"} ${suppressed ? "opacity-60" : ""}`}
    >
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <div className="flex flex-wrap gap-2">
            <span className="rounded-full border px-2.5 py-1 text-xs font-semibold">{finding.severity}</span>
            <span className="rounded-full border bg-muted px-2.5 py-1 text-xs">{finding.category}</span>
            {suppressed ? <span className="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2.5 py-1 text-xs text-emerald-700">Ignored for 7 days</span> : null}
          </div>
          <h3 className="mt-3 font-semibold">{finding.title}</h3>
        </div>
        <Button size="sm" variant="ghost" onClick={() => onIgnore(finding.id)}>{suppressed ? "Stop ignoring" : "Ignore 7 days"}</Button>
      </div>
      <details className="mt-3 rounded-lg border bg-muted/20 p-3">
        <summary className="cursor-pointer text-sm font-medium">View evidence and recommendation</summary>
        <div className="mt-3 space-y-3">
          <div>
            <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Evidence</p>
            <code className="mt-1 block break-words rounded-md bg-background p-2 text-xs">{finding.evidence}</code>
            <Button size="sm" variant="ghost" className="mt-1" onClick={() => copy("evidence", finding.evidence)}><Clipboard /> {copied === "evidence" ? "Copied" : "Copy evidence"}</Button>
          </div>
          <div>
            <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Recommendation</p>
            <p className="mt-1 text-sm leading-relaxed">{finding.recommendation}</p>
            <Button size="sm" variant="ghost" className="mt-1" onClick={() => copy("recommendation", finding.recommendation)}><Clipboard /> {copied === "recommendation" ? "Copied" : "Copy recommendation"}</Button>
          </div>
        </div>
      </details>
    </article>
  );
});

