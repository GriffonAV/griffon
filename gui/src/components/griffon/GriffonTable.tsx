import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { TableElement } from "@/components/types";

type Props = {
  element: TableElement;
};

export default function GriffonTable({ element }: Props) {
  return (
    <div id={element.id} className="w-full overflow-x-auto rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            {element.columns?.map((column) => (
              <TableHead key={column.key}>{column.label}</TableHead>
            ))}
          </TableRow>
        </TableHeader>

        <TableBody>
          {element.rows?.map((row, rowIndex) => (
            <TableRow key={row.id ?? rowIndex}>
              {element.columns?.map((column) => (
                <TableCell key={`${row.id ?? rowIndex}-${column.key}`}>
                  {String(row[column.key] ?? "")}
                </TableCell>
              ))}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}