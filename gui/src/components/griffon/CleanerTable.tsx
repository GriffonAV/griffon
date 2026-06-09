import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";
import { resolveFromPath } from "@/lib/utils";
import { Button } from "../ui/button";
import type { GriffonActionHandler } from "../types";

interface CleanerTableProps {
    element: {
        id: string;
        title?: string;
        from?: string;
        action?: string;
    };
    store?: Record<string, any>;
    onAction?: GriffonActionHandler;
}


type ModuleItem = {
    id: string;
    name: string;
    enabled: boolean;
    priority: number;
};

type ExpectedDataItem = {
    files_scanned: number;
    modules: ModuleItem[];
};


function handle_click(
    module: ModuleItem,
    element: any = {},
    onAction?: GriffonActionHandler,
    store: Record<string, any> = {}
) {
    if (!element?.action || !onAction) {
        return;
    }

    const currentSelected: ModuleItem[] =
        store?.data?.files_selected?.files ?? [];

    const exists = currentSelected.some(
        (item) => item.id === module.id
    );

    let nextSelected: ModuleItem[];

    if (exists) {
        nextSelected = currentSelected.filter(
            (item) => item.id !== module.id
        );
    } else {
        nextSelected = [...currentSelected, module];
    }

    onAction(element.action, {
        ...element,
        value: {
            files: nextSelected
        }
    });
}

export default function CleanerTable({
    element,
    store = {},
    onAction,
}: CleanerTableProps) {
    const column_names = ["id", "name", "enabled", "priority"];
    const context: { store: any; event?: any } = { store };
    const data: ExpectedDataItem = element.from ? resolveFromPath(element.from, context) : [];

    return (
        <>
            <div>
                <p>files scanned : {data?.files_scanned ?? 0}</p>
            </div>

            <div id={element.id} className="w-full overflow-x-auto rounded-md border">
                <Table>
                    <TableHeader>
                        <TableRow>
                            {column_names.map((column) => (
                                <TableHead key={column}>{column.charAt(0).toUpperCase() + column.slice(1)}</TableHead>
                            ))}
                        </TableRow>
                    </TableHeader>

                    <TableBody>
                        {data?.modules?.map((row, rowIndex) => {

                            const selected =
                                store?.data?.files_selected?.files?.some(
                                    (item: ModuleItem) => item.id === row.id
                                );

                            return (
                                <TableRow key={row.id ?? rowIndex}>
                                    {column_names.map((column) => (
                                        <TableCell key={`${row.id ?? rowIndex}-${column}`}>
                                            {String(row[column as keyof ModuleItem] ?? "")}
                                        </TableCell>
                                    ))}

                                    <TableCell>
                                        <Button
                                            onClick={() =>
                                                handle_click(
                                                    row,
                                                    element,
                                                    onAction,
                                                    store
                                                )
                                            }
                                        >
                                            {selected ? "Remove" : "Add"}
                                        </Button>
                                    </TableCell>
                                </TableRow>
                            );
                        })}
                    </TableBody>
                </Table>
            </div>
            <div>
                <pre>{JSON.stringify(store?.data?.files_selected ?? {}, null, 2)}</pre>
            </div>
        </>
    );
}