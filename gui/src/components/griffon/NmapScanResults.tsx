import { Badge } from "@/components/ui/badge";
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
};

function serviceDetails(port: NmapPort) {
  return [port.product, port.version, port.extra_info]
    .filter(Boolean)
    .join(" · ");
}

export default function NmapScanResults({ element, store = {} }: Props) {
  const result = resolveFromPath(element.from, { store }) as NmapScanResult | undefined;
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
                      <TableRow key={`${port.protocol}-${port.port}`}>
                        <TableCell className="font-mono">
                          {port.port}/{port.protocol}
                        </TableCell>
                        <TableCell>{port.service || "Unknown"}</TableCell>
                        <TableCell className="text-muted-foreground">
                          {serviceDetails(port) || "No version detected"}
                        </TableCell>
                      </TableRow>
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
