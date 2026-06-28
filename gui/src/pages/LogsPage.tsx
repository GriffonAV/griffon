import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { History, RefreshCw, ToyBrick } from "lucide-react";

import { NoPluginLayout } from "@/bindings/component/layout/NoPluginLayout";
import { PageLayout } from "@/components/layout/PageLayout";
import { Button } from "@/components/ui/button";

type PluginHistoryEntry = {
    timestamp: number;
    level: string;
    pluginName: string;
    pluginUuid: string;
    event?: string | null;
    pid?: string | null;
    path?: string | null;
    message: string;
    sourceFile: string;
};

function formatTimestamp(timestamp: number) {
    if (!timestamp) {
        return "-";
    }

    return new Intl.DateTimeFormat("fr-FR", {
        dateStyle: "short",
        timeStyle: "medium",
    }).format(new Date(timestamp * 1000));
}

function formatEvent(event?: string | null) {
    if (!event) {
        return "-";
    }

    return event.replaceAll("_", " ");
}

function getLevelClass(level: string) {
    switch (level.toUpperCase()) {
        case "INFO":
            return "text-green-500";
        case "WARN":
        case "WARNING":
            return "text-yellow-500";
        case "ERROR":
            return "text-red-500";
        default:
            return "text-muted-foreground";
    }
}

export default function LogsPage() {
    const [entries, setEntries] = useState<PluginHistoryEntry[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    async function loadPluginHistory() {
        try {
            setLoading(true);
            setError(null);

            const result = await invoke<PluginHistoryEntry[]>("get_plugin_history");
            setEntries(result);
        } catch (err) {
            console.error("Failed to load plugin history:", err);
            setError("Unable to load plugin history.");
        } finally {
            setLoading(false);
        }
    }

    useEffect(() => {
        loadPluginHistory();
    }, []);

    return (
        <PageLayout title="Activity Log">
            <NoPluginLayout>
                <section className="w-full rounded-xl border border-border bg-card p-6 shadow-sm">
                    <div className="flex flex-col gap-5">
                        <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                            <div className="flex items-center gap-3">
                                <History className="size-6" />

                                <div>
                                    <h2 className="text-xl font-bold">Plugin history</h2>

                                    <p className="mt-1 text-sm text-muted-foreground">
                                        Display plugin activity from Griffon history files.
                                    </p>
                                </div>
                            </div>

                            <Button
                                variant="outline"
                                className="cursor-pointer gap-2"
                                disabled={loading}
                                onClick={loadPluginHistory}
                            >
                                <RefreshCw className="size-4" />
                                {loading ? "Loading..." : "Refresh"}
                            </Button>
                        </div>

                        {error && (
                            <div className="rounded-md border border-red-500/40 bg-red-500/10 p-4 text-sm text-red-500">
                                {error}
                            </div>
                        )}

                        {!loading && !error && entries.length === 0 && (
                            <div className="rounded-md border border-border p-4 text-sm text-muted-foreground">
                                No plugin history found.
                            </div>
                        )}

                        {!error && entries.length > 0 && (
                            <div className="flex flex-col gap-3">
                                {entries.map((entry, index) => (
                                    <div
                                        key={`${entry.sourceFile}-${entry.timestamp}-${index}`}
                                        className="rounded-md border border-border p-4"
                                    >
                                        <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
                                            <div className="flex min-w-0 gap-3">
                                                <ToyBrick className="mt-1 shrink-0" />

                                                <div className="min-w-0">
                                                    <div className="flex flex-wrap items-center gap-2">
                                                        <p className="font-semibold">
                                                            {entry.pluginName}
                                                        </p>

                                                        <span
                                                            className={`text-xs font-medium ${getLevelClass(
                                                                entry.level,
                                                            )}`}
                                                        >
                                                            ● {entry.level}
                                                        </span>
                                                    </div>

                                                    <p className="mt-1 text-xs text-muted-foreground break-all">
                                                        {entry.pluginUuid}
                                                    </p>

                                                    <p className="mt-3 text-sm">
                                                        <span className="font-medium">Event:</span>{" "}
                                                        {formatEvent(entry.event)}
                                                    </p>

                                                    {entry.pid && (
                                                        <p className="mt-1 text-sm">
                                                            <span className="font-medium">PID:</span>{" "}
                                                            {entry.pid}
                                                        </p>
                                                    )}

                                                    {entry.path && (
                                                        <p className="mt-1 text-sm text-muted-foreground break-all">
                                                            <span className="font-medium text-foreground">
                                                                Path:
                                                            </span>{" "}
                                                            {entry.path}
                                                        </p>
                                                    )}

                                                    <p className="mt-3 rounded-md bg-muted p-3 text-xs break-all">
                                                        {entry.message}
                                                    </p>
                                                </div>
                                            </div>

                                            <div className="shrink-0 text-left text-xs text-muted-foreground sm:text-right">
                                                <p>{formatTimestamp(entry.timestamp)}</p>
                                            </div>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </section>
            </NoPluginLayout>
        </PageLayout>
    );
}