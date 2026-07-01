import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { ScannerTableElement } from "@/components/types";
import { resolveFromPath } from "@/lib/utils";

import GriffonCheckbox from "./GriffonCheckbox";

type Props = {
  element: ScannerTableElement;
  store: Record<string, any>;
};

export default function ScannerTable({ element, store = {} }: Props) {
  
  const rows =
    typeof element.rows === "string"
      ? resolveFromPath(element.rows, {store}) as any[] | undefined
      : [];

  console.log("rows", rows);
  
  return (
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
            <TableRow key={row.id ?? rowIndex}>
              {element.columns?.map((column) => (
                <TableCell key={`${row.id ?? rowIndex}-${column.key}`}>
                  {String(row[column.key] ?? "")}
                </TableCell>
              ))}
              <TableCell>
                <GriffonCheckbox
                  element={{
                    id: `${element.id}-checkbox-${rowIndex}`,
                    type: "checkbox",
                    label: "Select",
                    checked: !!store?.data?.selected_threats?.some(
                      (item: any) => item.id === row.id
                    )
                  }}
                  onAction={(action, updatedElement) => {
                    if (element.action) {
                      // Call the action handler with the updated element
                      console.log("Action triggered:", action, updatedElement);
                      // You can implement your action handling logic here
                    }
                  }}
                />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}