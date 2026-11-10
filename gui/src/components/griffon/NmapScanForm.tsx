import { ChevronDown, Container, Loader2, Radar, RefreshCw, ScanLine, SlidersHorizontal } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { resolveFromPath } from "@/lib/utils";

type Request = Record<string, string | boolean>;
type ContainerInfo = { name: string; image: string; status: string };
type Props = { element: { id: string; from?: string }; store: Record<string, unknown>; onAction?: (action: string, event?: { value?: string; checked?: boolean }) => Promise<void> | void };

const profiles = [
  ["quick", "Quick scan", "Common ports"], ["service", "Service discovery", "Identify versions"],
  ["web", "Web services", "80, 443, 8080, 8443"], ["full_tcp", "Full TCP", "All TCP ports"],
  ["discovery", "Host discovery", "No port scan"], ["udp", "UDP", "Targeted UDP scan"],
];

function Toggle({ label, description, checked, onChange }: { label: string; description: string; checked: boolean; onChange: (checked: boolean) => void }) {
  return <div className="flex items-start justify-between gap-4 rounded-lg border border-border/70 bg-muted/20 p-3"><div><p className="text-sm font-medium">{label}</p><p className="mt-1 text-xs text-muted-foreground">{description}</p></div><Switch checked={checked} onCheckedChange={onChange} /></div>;
}

function Choice({ value, onChange, options }: { value: string; onChange: (value: string) => void; options: Array<[string, string]> }) {
  return <Select value={value} onValueChange={onChange}><SelectTrigger className="w-full bg-card text-card-foreground"><SelectValue /></SelectTrigger><SelectContent>{options.map(([optionValue, label]) => <SelectItem key={optionValue} value={optionValue}>{label}</SelectItem>)}</SelectContent></Select>;
}

export default function NmapScanForm({ element, store, onAction }: Props) {
  const [isScanning, setIsScanning] = useState(false);
  const request = (resolveFromPath(element.from, { store }) ?? {}) as Request;
  const exegol = (resolveFromPath("store.data.exegol_containers", { store }) ?? {}) as { message?: string; containers?: ContainerInfo[] };
  const set = (action: string, value: string | boolean) => onAction?.(action, typeof value === "boolean" ? { checked: value } : { value });
  const selectedProfile = String(request.profile ?? "quick");
  const runScan = async () => {
    setIsScanning(true);
    try { await onAction?.("nmap.run"); } finally { setIsScanning(false); }
  };

  return <div id={element.id} className="mx-auto w-full max-w-5xl space-y-5">
    <div className="rounded-2xl border border-primary/20 bg-gradient-to-br from-primary/12 via-background to-background p-6">
      <div className="flex items-start gap-4"><div className="rounded-xl bg-primary p-3 text-primary-foreground"><Radar className="size-6" /></div><div><h1 className="text-2xl font-semibold tracking-tight">Network reconnaissance</h1><p className="mt-1 text-sm text-muted-foreground">Build a focused Nmap scan and inspect the services discovered on an authorised target.</p></div></div>
      <div className="mt-6 flex gap-3"><Input value={String(request.target ?? "")} onChange={(event) => set("nmap.set_target", event.target.value)} placeholder="Target IP or hostname" className="h-11 bg-background" /><Button className="h-11 shrink-0" disabled={isScanning} onClick={runScan}>{isScanning ? <><Loader2 className="animate-spin" /> Scan in progress…</> : <><ScanLine /> Start scan</>}</Button></div>
      {isScanning && <div className="mt-3 flex items-center gap-2 text-sm text-primary"><span className="size-2 animate-pulse rounded-full bg-primary" />Nmap is running. Results will appear when the scan completes.</div>}
    </div>

    <Card><CardContent className="pt-6"><div className="mb-3 flex items-center gap-2 text-sm font-semibold"><ScanLine className="size-4 text-primary" /> Scan profile</div><div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">{profiles.map(([value, title, description]) => <button key={value} type="button" onClick={() => set("nmap.set_profile", value)} className={`rounded-xl border p-4 text-left transition-colors ${selectedProfile === value ? "border-primary bg-primary/10" : "border-border hover:bg-muted/50"}`}><p className="font-medium">{title}</p><p className="mt-1 text-xs text-muted-foreground">{description}</p></button>)}</div></CardContent></Card>

    <div className="grid gap-5 lg:grid-cols-[1.25fr_0.75fr]">
      <Card><CardContent className="space-y-4 pt-6"><div className="flex items-center gap-2 text-sm font-semibold"><Container className="size-4 text-primary" /> Execution environment</div><label className="text-xs font-medium text-muted-foreground">Run Nmap from</label><Choice value={String(request.executor ?? "local")} onChange={(value) => set("nmap.set_executor", value)} options={[["local", "This computer"], ["exegol", "Exegol container"]]} />{request.executor === "exegol" && <><div className="flex gap-2"><Input value={String(request.exegol_container ?? "")} onChange={(event) => set("nmap.set_exegol_container", event.target.value)} placeholder="Container name, e.g. htb" /><Button variant="outline" onClick={() => onAction?.("nmap.refresh_exegol")}><RefreshCw /> Refresh</Button></div>{exegol.message && <p className="text-xs text-muted-foreground">{exegol.message}</p>}{exegol.containers?.map((container) => <button key={container.name} type="button" onClick={() => set("nmap.set_exegol_container", container.name)} className="flex w-full items-center justify-between rounded-lg border border-border p-3 text-left hover:bg-muted/50"><span><span className="block text-sm font-medium">{container.name}</span><span className="text-xs text-muted-foreground">{container.image}</span></span><span className="text-xs text-muted-foreground">{container.status}</span></button>)}</>}</CardContent></Card>
      <Card><CardContent className="space-y-4 pt-6"><div className="flex items-center gap-2 text-sm font-semibold"><SlidersHorizontal className="size-4 text-primary" /> Core options</div><label className="block text-xs font-medium text-muted-foreground">Timing</label><Choice value={String(request.timing ?? "t4")} onChange={(value) => set("nmap.set_timing", value)} options={[["t3", "T3 — Conservative"], ["t4", "T4 — Fast"]]} /><label className="block text-xs font-medium text-muted-foreground">TCP method</label><Choice value={String(request.scan_type ?? "auto")} onChange={(value) => set("nmap.set_scan_type", value)} options={[["auto", "Automatic"], ["connect", "TCP connect"], ["syn", "SYN — requires privileges"]]} /><Input value={String(request.ports ?? "")} onChange={(event) => set("nmap.set_ports", event.target.value)} placeholder="Custom ports: 22,80,443" /></CardContent></Card>
    </div>

    <details className="rounded-xl border border-border bg-card p-4"><summary className="flex cursor-pointer list-none items-center justify-between font-medium">Advanced options <ChevronDown className="size-4 text-muted-foreground" /></summary><div className="mt-4 grid gap-3 md:grid-cols-2"><Toggle label="Skip host discovery" description="Use -Pn for known hosts that do not answer probes." checked={Boolean(request.skip_host_discovery)} onChange={(value) => set("nmap.set_skip_host_discovery", value)} /><Toggle label="Service versions" description="Identify the software behind open ports." checked={Boolean(request.version_detection)} onChange={(value) => set("nmap.set_version_detection", value)} /><Toggle label="Operating system" description="Requires privileges and suitable port responses." checked={Boolean(request.os_detection)} onChange={(value) => set("nmap.set_os_detection", value)} /><Toggle label="Default NSE scripts" description="Run Nmap's standard discovery scripts." checked={Boolean(request.default_scripts)} onChange={(value) => set("nmap.set_default_scripts", value)} /></div></details>
  </div>;
}
