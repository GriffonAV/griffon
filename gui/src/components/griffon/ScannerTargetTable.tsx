import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { ScannerTargetTableElement } from "@/components/types";
import { Trash } from 'lucide-react'
import { resolveFromPath } from "@/lib/utils";
import { Button } from "../ui/button";

type Props = {
  element: ScannerTargetTableElement;
  store: Record<string, any>;
  onAction?: (action: string, event?: any) => void;
};


export default function ScannerTargetTable({ element, store = {}, onAction }: Props) {
  const rows: String[] = resolveFromPath(element.targets, {store}) ?? [];

  return (
    <div id={element.id} className="w-full overflow-x-auto rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead key="path">path</TableHead>
            <TableHead key="select">remove from list</TableHead>
          </TableRow>
        </TableHeader>

        <TableBody>
          {rows?.map((row, rowIndex) => (
            <TableRow key={rowIndex}>
                <TableCell key={`${rowIndex}`}>
                  {String(row ?? "")}
                </TableCell>
              <TableCell>
                <Button
                  onClick={() => {
                    onAction?.(element.action ?? "", {
                      value: row,
                      append: false,
                    });
                  }}
                ><Trash/></Button>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}