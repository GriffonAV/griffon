import { AlertCircle, CheckCircle2, Info, Trash2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { resolveFromPath } from "@/lib/utils";

interface CleanerDeleteResultProps {
    element: {
        id: string;
        title?: string;
        from?: string;
    };
    store?: Record<string, any>;
}

type DeleteFailedItem = {
    path: string;
    error: string;
};

type DeleteResult = {
    ok: boolean;
    dry_run: boolean;
    deleted_count: number;
    deleted_bytes: number;
    failed: DeleteFailedItem[];
};

function formatBytes(bytes: number): string {
    if (!bytes || bytes <= 0) {
        return "0 B";
    }

    const units = ["B", "KB", "MB", "GB", "TB"];
    const index = Math.floor(Math.log(bytes) / Math.log(1024));
    const value = bytes / Math.pow(1024, index);

    return `${value.toFixed(value >= 10 ? 1 : 2)} ${units[index]}`;
}

export default function CleanerDeleteResult({
                                                element,
                                                store = {},
                                            }: CleanerDeleteResultProps) {
    const context: { store: any; event?: any } = { store };

    const result: DeleteResult | undefined = element.from
        ? resolveFromPath(element.from, context)
        : undefined;

    if (!result) {
        return null;
    }

    const hasResult =
        result.deleted_count > 0 ||
        result.deleted_bytes > 0 ||
        (result.failed?.length ?? 0) > 0;

    if (!hasResult) {
        return null;
    }

    const failedCount = result.failed?.length ?? 0;

    return (
        <Card id={element.id} className="w-full">
            <CardContent className="space-y-4 p-4">
                <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                    <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-md bg-muted">
                            {result.ok ? (
                                <CheckCircle2 className="h-5 w-5 text-green-600" />
                            ) : (
                                <AlertCircle className="h-5 w-5 text-red-600" />
                            )}
                        </div>

                        <div>
                            <h3 className="font-semibold">
                                {element.title ??
                                    (result.dry_run
                                        ? "Dry run result"
                                        : "Cleanup result")}
                            </h3>

                            <p className="text-sm text-muted-foreground">
                                {result.dry_run
                                    ? "No file was deleted. This is only a simulation."
                                    : "The selected cleanup operation has completed."}
                            </p>
                        </div>
                    </div>

                    <div className="flex flex-wrap gap-2">
                        <Badge variant={result.ok ? "default" : "destructive"}>
                            {result.ok ? "Success" : "Partial failure"}
                        </Badge>

                        {result.dry_run && (
                            <Badge variant="outline">Dry run</Badge>
                        )}
                    </div>
                </div>

                <div className="grid gap-3 md:grid-cols-3">
                    <div className="rounded-md border bg-muted/30 p-3">
                        <div className="flex items-center gap-2 text-sm text-muted-foreground">
                            <Trash2 className="h-4 w-4" />
                            Selected items
                        </div>

                        <p className="mt-1 text-2xl font-semibold">
                            {result.deleted_count}
                        </p>

                        <p className="text-xs text-muted-foreground">
                            {result.dry_run
                                ? "item(s) would be deleted"
                                : "item(s) deleted"}
                        </p>
                    </div>

                    <div className="rounded-md border bg-muted/30 p-3">
                        <div className="flex items-center gap-2 text-sm text-muted-foreground">
                            <Info className="h-4 w-4" />
                            Reclaimable size
                        </div>

                        <p className="mt-1 text-2xl font-semibold">
                            {formatBytes(result.deleted_bytes)}
                        </p>

                        <p className="text-xs text-muted-foreground">
                            {result.dry_run
                                ? "would be freed"
                                : "freed from disk"}
                        </p>
                    </div>

                    <div className="rounded-md border bg-muted/30 p-3">
                        <div className="flex items-center gap-2 text-sm text-muted-foreground">
                            <AlertCircle className="h-4 w-4" />
                            Failed items
                        </div>

                        <p className="mt-1 text-2xl font-semibold">
                            {failedCount}
                        </p>

                        <p className="text-xs text-muted-foreground">
                            error(s) reported
                        </p>
                    </div>
                </div>

                {failedCount > 0 && (
                    <div className="space-y-2 rounded-md border border-destructive/40 bg-destructive/5 p-3">
                        <p className="text-sm font-medium text-destructive">
                            Failed paths
                        </p>

                        <div className="space-y-2">
                            {result.failed.map((item, index) => (
                                <div
                                    key={`${item.path}-${index}`}
                                    className="rounded-md border bg-background p-2"
                                >
                                    <p className="truncate text-xs font-medium">
                                        {item.path}
                                    </p>

                                    <p className="text-xs text-muted-foreground">
                                        {item.error}
                                    </p>
                                </div>
                            ))}
                        </div>
                    </div>
                )}

                <div className="rounded-md border bg-muted/20 p-3">
                    <p className="mb-2 text-sm font-medium">Raw result</p>

                    <pre className="max-h-[180px] overflow-auto text-xs text-muted-foreground">
                        {JSON.stringify(result, null, 2)}
                    </pre>
                </div>
            </CardContent>
        </Card>
    );
}