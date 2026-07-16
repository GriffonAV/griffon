import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { TableElement, TableRowData } from "@/components/types";
import { resolveFromPath } from "@/lib/utils";
import { Checkbox } from "../ui/checkbox";

type Props = {
  element: TableElement;
  store: Record<string, any>;
  onAction?: (action: string, event?: any) => void;
};

function getFirstTransmitValue(element: TableElement, rows: TableRowData[], rowID: number) {
  if (element.columns == null)
    return;

  var value_arr: Array<any> = []
  console.log("rowID: " + rowID);
  console.log("rows: " + JSON.stringify(rows));
  console.log("element.columns: " + JSON.stringify(element.columns));
  for (let i = 0; i < element.columns.length; i++) {
    if (element.columns[i].transmit) {
      value_arr.push(rows[rowID][element.columns[i].key]);
      console.log("transmit value: " + rows[rowID][element.columns[i].key]);
    }
  }
  console.log("value_arr: " + JSON.stringify(value_arr));
  return value_arr;
}


export default function GriffonTable({ element, store = {}, onAction }: Props) {
  const store_data = element.from ? resolveFromPath(element.from, { store }) : null;
  const rows: TableRowData[] = store_data ? store_data : (element.rows ?? [])

  return (
    <div id={element.id} className="w-full overflow-x-auto rounded-md border">
      {/* display store */}
      {/* {element.from && (
        <div className="p-2 text-sm text-gray-500">
          <pre>{JSON.stringify(store_data, null, 2)}</pre>
        </div>
      )} */}
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead key="select">Select</TableHead>
            {element.columns?.map((column) => (
              column.hide ? <></> : <TableHead key={column.key}>{column.label}</TableHead>
            ))}
          </TableRow>
        </TableHeader>

        <TableBody>
          {rows.map((row, rowIndex) => (
            <TableRow key={rowIndex}>
              <TableCell>
                <Checkbox
                  onCheckedChange={(checked) => {
                    onAction?.(element.action ?? "", {
                      value: getFirstTransmitValue(element, rows, rowIndex),
                      checked: checked,
                    });
                  }}
                />
              </TableCell>
              {element.columns?.map((column) => (
                column.hide ? <></> :
                  <TableCell key={`${rowIndex}-${column.key}`}>
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