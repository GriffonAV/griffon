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
import { FileText, FolderSearch, Globe, KeyRound, ListTree, Mail, Network } from "lucide-react";

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

type ServiceAction = {
  id: string;
  label: string;
  icon: typeof KeyRound;
};

function actionsForService(service: string): ServiceAction[] {
  switch (service) {
    case "ssh":
      return [{ id: "ssh_keys", label: "SSH keys", icon: KeyRound }];
    case "ftp":
      return [{ id: "ftp_anonymous", label: "Anonymous FTP", icon: FolderSearch }];
    case "smtp":
    case "submission":
    case "smtps":
      return [{ id: "smtp_capabilities", label: "SMTP capabilities", icon: Mail }];
    case "domain":
    case "dns":
      return [{ id: "dns_identification", label: "DNS identification", icon: Network }];
    case "microsoft-ds":
    case "netbios-ssn":
    case "smb":
      return [{ id: "smb_shares", label: "SMB shares", icon: FolderSearch }];
    default:
      return [];
  }
}

type ProbeResult = {
  target?: string;
  port?: string;
  message?: string;
  output?: string;
  command?: string;
};

export default function NmapScanResults({ element, store = {}, onAction }: Props) {
  const result = resolveFromPath(element.from, { store }) as NmapScanResult | undefined;
  const request = resolveFromPath("store.data.scan_request", { store }) as Record<string, string> | undefined;
  const probe = resolveFromPath("store.data.http_probe", { store }) as ProbeResult | undefined;
  const serviceProbe = resolveFromPath("store.data.service_probe", { store }) as ProbeResult | undefined;
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
                    {host.ports.map((port) => {
                      const serviceActions = actionsForService(port.service);
                      const isHttp = ["http", "https"].includes(port.service);
                      const matchingHttpProbe = probe?.target === host.address && probe.port === port.port;
                      const matchingServiceProbe = serviceProbe?.target === host.address && serviceProbe.port === port.port;

                      return (
                        <TableRow key={`${port.protocol}-${port.port}`}>
                          <TableCell className="font-mono">{port.port}/{port.protocol}</TableCell>
                          <TableCell>{port.service || "Unknown"}</TableCell>
                          <TableCell className="text-muted-foreground">
                            {serviceDetails(port) || "No version detected"}
                            {(isHttp || serviceActions.length > 0) && (
                              <div className="mt-3 flex flex-wrap gap-2">
                                {isHttp && (
                                  <>
                                    <Button size="sm" variant="outline" onClick={() => onAction?.("nmap.http_probe", { value: { target: host.address, port: port.port, protocol: port.service, probe: "headers", executor: request?.executor ?? "local", exegol_container: request?.exegol_container ?? "" } })}><Globe /> Headers</Button>
                                    <Button size="sm" variant="outline" onClick={() => onAction?.("nmap.http_probe", { value: { target: host.address, port: port.port, protocol: port.service, probe: "title", executor: request?.executor ?? "local", exegol_container: request?.exegol_container ?? "" } })}><FileText /> Title</Button>
                                    <Button size="sm" variant="outline" onClick={() => onAction?.("nmap.http_probe", { value: { target: host.address, port: port.port, protocol: port.service, probe: "robots", executor: request?.executor ?? "local", exegol_container: request?.exegol_container ?? "" } })}><ListTree /> robots.txt</Button>
                                  </>
                                )}
                                {serviceActions.map((action) => {
                                  const Icon = action.icon;
                                  return <Button key={action.id} size="sm" variant="outline" onClick={() => onAction?.("nmap.service_probe", { value: { target: host.address, port: port.port, service: port.service, action: action.id, executor: request?.executor ?? "local", exegol_container: request?.exegol_container ?? "" } })}><Icon /> {action.label}</Button>;
                                })}
                              </div>
                            )}
                            {matchingHttpProbe && probe.message && <div className="mt-3 rounded-md bg-muted p-3 text-xs text-foreground"><p className="mb-2 font-medium">{probe.message}</p>{probe.output && <pre className="max-h-48 overflow-auto whitespace-pre-wrap font-mono">{probe.output}</pre>}</div>}
                            {matchingServiceProbe && serviceProbe.message && <div className="mt-3 rounded-md bg-muted p-3 text-xs text-foreground"><p className="mb-2 font-medium">{serviceProbe.message}</p>{serviceProbe.output && <pre className="max-h-48 overflow-auto whitespace-pre-wrap font-mono">{serviceProbe.output}</pre>}</div>}
                          </TableCell>
                        </TableRow>
                      );
                    })}
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
