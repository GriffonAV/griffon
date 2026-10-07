import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { NmapScanResultsElement } from "@/components/types";
import { resolveFromPath } from "@/lib/utils";
import { FileText, Globe, ListTree } from "lucide-react";

type NmapPort = {
  port: string;
  protocol: string;
  state: string;
  service: string;
  product: string;
  version: string;
  extra_info: string;
};

type NmapHost = {
  address: string;
  hostname: string;
  state: string;
  ports: NmapPort[];
};

type NmapScanResult = {
  hosts?: NmapHost[];
};

type Props = {
  element: NmapScanResultsElement;
  store: Record<string, unknown>;
  onAction?: (action: string, event?: { value: unknown }) => void;
};

function serviceDetails(port: NmapPort) {
  return [port.product, port.version, port.extra_info]
    .filter(Boolean)
    .join(" · ");
}

export default function NmapScanResults({ element, store = {}, onAction }: Props) {
  const result = resolveFromPath(element.from, { store }) as NmapScanResult | undefined;
  const request = resolveFromPath("store.data.scan_request", { store }) as Record<string, string> | undefined;
  const probe = resolveFromPath("store.data.http_probe", { store }) as { target?: string; port?: string; message?: string; output?: string; command?: string } | undefined;
  const hosts = Array.isArray(result?.hosts) ? result.hosts : [];

  if (hosts.length === 0) {
    return null;
  }

  return (
    <div id={element.id} className="space-y-4">
      <h2 className="text-lg font-semibold">Open services</h2>
      {hosts.map((host, hostIndex) => (
        <Card key={`${host.address}-${hostIndex}`}>
          <CardHeader className="pb-0">
            <div className="flex items-center justify-between gap-4">
              <div>
                <CardTitle>{host.address}</CardTitle>
                <CardDescription>
                  {host.hostname || "No hostname returned by Nmap"}
                </CardDescription>
              </div>
              <Badge variant={host.state === "up" ? "default" : "secondary"}>
                {host.state}
              </Badge>
            </div>
          </CardHeader>
          <CardContent>
            {host.ports.length > 0 ? (
              <div className="overflow-x-auto rounded-md border">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Port</TableHead>
                      <TableHead>Service</TableHead>
                      <TableHead>Product and version</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {host.ports.map((port) => (
                      <TableRow key={`${port.protocol}-${port.port}`}><TableCell className="font-mono">{port.port}/{port.protocol}</TableCell><TableCell>{port.service || "Unknown"}</TableCell><TableCell className="text-muted-foreground">{serviceDetails(port) || "No version detected"}{["http", "https"].includes(port.service) && <div className="mt-3 flex flex-wrap gap-2"><Button size="sm" variant="outline" onClick={() => onAction?.("nmap.http_probe", { value: { target: host.address, port: port.port, protocol: port.service, probe: "headers", executor: request?.executor ?? "local", exegol_container: request?.exegol_container ?? "" } })}><Globe /> Headers</Button><Button size="sm" variant="outline" onClick={() => onAction?.("nmap.http_probe", { value: { target: host.address, port: port.port, protocol: port.service, probe: "title", executor: request?.executor ?? "local", exegol_container: request?.exegol_container ?? "" } })}><FileText /> Title</Button><Button size="sm" variant="outline" onClick={() => onAction?.("nmap.http_probe", { value: { target: host.address, port: port.port, protocol: port.service, probe: "robots", executor: request?.executor ?? "local", exegol_container: request?.exegol_container ?? "" } })}><ListTree /> robots.txt</Button></div>}{probe?.target === host.address && probe.port === port.port && probe.output && <div className="mt-3 rounded-md bg-muted p-3 text-xs text-foreground"><p className="mb-2 font-medium">{probe.message}</p><pre className="max-h-48 overflow-auto whitespace-pre-wrap font-mono">{probe.output}</pre></div>}</TableCell></TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">
                No open ports were found with this profile.
              </p>
            )}
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
