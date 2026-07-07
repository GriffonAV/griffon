import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { ScannerTableElement, Threat } from "@/components/types";
import { Checkbox } from "../ui/checkbox";
import { resolveFromPath } from "@/lib/utils";

type Props = {
  element: ScannerTableElement;
  store: Record<string, any>;
  onAction?: (action: string, event?: any) => void;
};

type ScanData = {
  total_scanned: number;
  total_skipped: number;
  total_errors: number;
  total_threats: number;
  threats: [Threat];
};

export default function ScannerTable({ element, store = {}, onAction }: Props) {
  var scan_result = resolveFromPath(element.scan_result, {store})
  var scanData: ScanData = scan_result ? JSON.parse(scan_result) : {};
  const rows: Threat[] = scanData.threats;

  return (
    <div>
      {scanData.total_scanned > 0 && (
        <div className="flex items-center gap-6 text-sm text-muted-foreground mb-4">
          <div>
            <span className="font-medium text-foreground">Files scanned:</span>{" "}
            {scanData.total_scanned}
          </div>

          <div>
            <span className="font-medium text-foreground">Threats found:</span>{" "}
            {scanData.total_threats}
          </div>
        </div>
      )}
      <div id={element.id} className="w-full overflow-x-auto rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              {element.columns?.map((column) => (
                <TableHead key={column.key}>{column.label}</TableHead>
              ))}
              <TableHead key="select">Select</TableHead>
            </TableRow>
          </TableHeader>

          <TableBody>
            {rows?.map((row, rowIndex) => (
              <TableRow key={rowIndex}>
                {element.columns?.map((column) => (
                  <TableCell key={`${rowIndex}-${column.key}`}>
                    {String(row[column.key] ?? "")}
                  </TableCell>
                ))}
                <TableCell>
                  <Checkbox
                    onCheckedChange={(checked) => {
                      onAction?.(element.action ?? "", {
                        value: row.path,
                        checked: checked,
                      });
                    }}
                  />
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>

  );
}