import { History, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { resolveFromPath } from "@/lib/utils";

type Scan = { id: string; created_at_ms: number; target: string; profile: string; executor: string; duration_ms: number; hosts: Array<{ ports: unknown[] }> };
type Props = { element: { id: string; from?: string }; store: Record<string, unknown>; onAction?: (action: string) => void };

export default function NmapHistory({ element, store, onAction }: Props) {
  const data = (resolveFromPath(element.from, { store }) ?? {}) as { message?: string; scans?: Scan[] };
  const scans = data.scans ?? [];
  return <div id={element.id} className="mx-auto w-full max-w-5xl space-y-4"><div className="flex items-center justify-between rounded-2xl border bg-card p-5"><div className="flex items-center gap-3"><div className="rounded-xl bg-primary/10 p-3 text-primary"><History /></div><div><h1 className="text-xl font-semibold">Scan history</h1><p className="text-sm text-muted-foreground">{data.message || "Saved Nmap scans"}</p></div></div><Button variant="outline" onClick={() => onAction?.("nmap.refresh_history")}><RefreshCw /> Refresh</Button></div>{scans.length === 0 ? <Card><CardContent className="py-10 text-center text-sm text-muted-foreground">No completed scans have been saved yet.</CardContent></Card> : scans.map((scan) => <Card key={scan.id}><CardContent className="flex items-center justify-between gap-4 py-5"><div><p className="font-mono font-medium">{scan.target}</p><p className="mt-1 text-sm text-muted-foreground">{new Date(scan.created_at_ms).toLocaleString()} · {scan.profile} · {scan.executor}</p></div><div className="text-right"><p className="font-medium">{scan.hosts.reduce((count, host) => count + host.ports.length, 0)} open ports</p><p className="text-sm text-muted-foreground">{(scan.duration_ms / 1000).toFixed(1)} s</p></div></CardContent></Card>)}</div>;
}
