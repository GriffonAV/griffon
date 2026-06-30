import { useEffect, useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { History, RefreshCw, ToyBrick, Filter } from "lucide-react";

import { NoPluginLayout } from "@/bindings/component/layout/NoPluginLayout";
import { PageLayout } from "@/components/layout/PageLayout";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

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

    // --- New Filter State ---
    const [levelFilter, setLevelFilter] = useState<string>("ALL");
    const [pluginFilter, setPluginFilter] = useState<string>("ALL");

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

    // --- Compute Unique Plugins for the Dropdown ---
    const uniquePlugins = useMemo(() => {
        const plugins = new Set(entries.map((e) => e.pluginName));
        return Array.from(plugins).sort();
    }, [entries]);

    // --- Apply Filters to Entries ---
    const filteredEntries = useMemo(() => {
        return entries.filter((entry) => {
            // Check Level Match (Normalize to uppercase, treat WARN/WARNING identically)
            const entryLevel = entry.level.toUpperCase().startsWith("WARN") ? "WARN" : entry.level.toUpperCase();
            const matchLevel = levelFilter === "ALL" || entryLevel === levelFilter;

            // Check Plugin Match
            const matchPlugin = pluginFilter === "ALL" || entry.pluginName === pluginFilter;

            return matchLevel && matchPlugin;
        });
    }, [entries, levelFilter, pluginFilter]);

    return (
        <PageLayout title="Activity Log">
            <NoPluginLayout>
                <section className="flex flex-col h-[calc(100vh-8.5rem)] w-full rounded-md border border-border bg-card p-6 shadow-sm">

                    {/* Header */}
                    <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between shrink-0 mb-6">
                        <div className="flex items-center gap-3">
                            <History className="size-6 text-foreground" />
                            <div>
                                <h2 className="text-lg font-semibold">Plugin history</h2>
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
                            <RefreshCw className={`size-4 ${loading ? "animate-spin" : ""}`} />
                            {loading ? "Loading..." : "Refresh"}
                        </Button>
                    </div>

                    {error && (
                        <div className="shrink-0 mb-4 rounded-md border border-red-500/40 bg-red-500/10 p-4 text-sm text-red-500">
                            {error}
                        </div>
                    )}

                    {!loading && !error && entries.length === 0 && (
                        <div className="shrink-0 rounded-md border border-border p-4 text-sm text-muted-foreground">
                            No plugin history found.
                        </div>
                    )}

                    {/* --- Filter Bar --- */}
                    {!error && entries.length > 0 && (
                        <div className="shrink-0 flex flex-col sm:flex-row gap-4 mb-4 pb-4 border-b border-border">
                            {/* Level Toggle Group */}
                            <div className="flex items-center gap-2">
                                <Filter className="size-4 text-muted-foreground" />
                                <span className="text-sm font-medium text-muted-foreground mr-1">Level:</span>

                                {/* Container keeps bg-muted */}
                                <div className="flex bg-muted p-1 rounded-md gap-1">
                                    {["ALL", "INFO", "WARN", "ERROR"].map((lvl) => (
                                        <Button
                                            key={lvl}
                                            variant="ghost" // Base it on ghost to strip default borders/backgrounds
                                            size="sm"
                                            className={`h-7 px-3 text-xs cursor-pointer transition-all ${levelFilter === lvl
                                                ? "bg-background text-foreground shadow-sm hover:bg-background" // Active state: raised tab
                                                : "text-muted-foreground hover:bg-transparent hover:text-foreground" // Inactive state: blends in
                                                }`}
                                            onClick={() => setLevelFilter(lvl)}
                                        >
                                            {lvl === "WARN" ? "WARNING" : lvl}
                                        </Button>
                                    ))}
                                </div>
                            </div>

                            {/* Plugin Select Dropdown */}
                            <div className="flex items-center gap-2">
                                <span className="text-sm font-medium text-muted-foreground mr-1">Plugin:</span>
                                <Select
                                    value={pluginFilter}
                                    onValueChange={(value) => setPluginFilter(value)}
                                >
                                    <SelectTrigger className="h-9 rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring cursor-pointer">
                                        <SelectValue placeholder="All Plugins" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="ALL">All Plugins</SelectItem>
                                        {uniquePlugins.map((plugin) => (
                                            <SelectItem key={plugin} value={plugin}>
                                                {plugin}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                        </div>
                    )}

                    {/* No matches for filters state */}
                    {!loading && !error && entries.length > 0 && filteredEntries.length === 0 && (
                        <div className="shrink-0 rounded-md border border-border p-4 text-sm text-muted-foreground text-center">
                            No logs match the selected filters.
                        </div>
                    )}

                    {/* The Logs Container */}
                    {!error && filteredEntries.length > 0 && (
                        <div className="flex-1 overflow-y-auto pr-2 flex flex-col gap-4 min-h-0">
                            {filteredEntries.map((entry, index) => (
                                <div
                                    key={`${entry.sourceFile}-${entry.timestamp}-${index}`}
                                    className="rounded-md border border-border p-4 shrink-0"
                                >
                                    <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
                                        <div className="flex min-w-0 gap-3">
                                            <ToyBrick className="mt-1 shrink-0 text-muted-foreground size-5" />

                                            <div className="min-w-0">
                                                <div className="flex flex-wrap items-center gap-2">
                                                    <p className="text-sm font-semibold">
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

                                                <p className="mt-4 rounded-md bg-muted p-3 text-xs break-all text-foreground">
                                                    {entry.message}
                                                </p>
                                            </div>
                                        </div>

                                        <div className="shrink-0 text-left text-xs text-muted-foreground sm:text-right mt-2 sm:mt-0">
                                            <p>{formatTimestamp(entry.timestamp)}</p>
                                        </div>
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}
                </section>
            </NoPluginLayout>
        </PageLayout>
    );
}