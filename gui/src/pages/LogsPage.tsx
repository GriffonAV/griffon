import { useEffect, useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { History, RefreshCw, Filter, ChevronRight, ChevronDown } from "lucide-react";

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
    if (!timestamp) return "-";
    return new Intl.DateTimeFormat("fr-FR", {
        dateStyle: "short",
        timeStyle: "medium",
    }).format(new Date(timestamp * 1000));
}

function formatEvent(event?: string | null) {
    if (!event) return "-";
    return event.replaceAll("_", " ");
}

function getLevelClass(level: string) {
    switch (level.toUpperCase()) {
        case "INFO": return "text-green-500";
        case "WARN":
        case "WARNING": return "text-yellow-500";
        case "ERROR": return "text-red-500";
        default: return "text-muted-foreground";
    }
}

function LogEntryRow({ entry, isAllTab }: { entry: PluginHistoryEntry; isAllTab: boolean }) {
    const [expanded, setExpanded] = useState(false);

    return (
        <div
            className="flex flex-col border-b border-border/50 hover:bg-muted/30 px-2 py-1.5 transition-colors cursor-pointer"
            onClick={() => setExpanded(!expanded)}
        >
            <div className="flex items-center gap-3 min-w-0">
                <div className="shrink-0 text-muted-foreground">
                    {expanded ? <ChevronDown className="size-4" /> : <ChevronRight className="size-4" />}
                </div>

                <span className="text-xs text-muted-foreground whitespace-nowrap shrink-0 w-32">
                    {formatTimestamp(entry.timestamp)}
                </span>

                <span className={`text-xs font-bold w-16 shrink-0 ${getLevelClass(entry.level)}`}>
                    {entry.level}
                </span>

                {isAllTab && (
                    <span className="text-xs font-semibold w-24 truncate shrink-0 text-foreground">
                        {entry.pluginName}
                    </span>
                )}

                {/* Single-line message preview */}
                <span className="text-xs truncate flex-1 text-foreground/80 font-mono">
                    {entry.event && <span className="text-muted-foreground mr-2">[{formatEvent(entry.event)}]</span>}
                    {entry.message}
                </span>
            </div>

            {expanded && (
                <div className="mt-2 mb-1 ml-7 flex flex-col gap-1 text-xs bg-muted/40 p-3 rounded-md border border-border/50">
                    <p><strong className="text-foreground">Message:</strong> <span className="font-mono text-muted-foreground">{entry.message}</span></p>
                    {entry.path && <p><strong className="text-foreground">Path:</strong> <span className="text-muted-foreground break-all">{entry.path}</span></p>}
                    {entry.pid && <p><strong className="text-foreground">PID:</strong> <span className="text-muted-foreground">{entry.pid}</span></p>}
                    <p><strong className="text-foreground">UUID:</strong> <span className="text-muted-foreground">{entry.pluginUuid}</span></p>
                </div>
            )}
        </div>
    );
}

export default function LogsPage() {
    const [entries, setEntries] = useState<PluginHistoryEntry[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const [levelFilter, setLevelFilter] = useState<string>("ALL");

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

    const uniquePlugins = useMemo(() => {
        const plugins = new Set(entries.map((e) => e.pluginName));
        return Array.from(plugins).sort();
    }, [entries]);

    const tabsList = useMemo(() => ["All", ...uniquePlugins], [uniquePlugins]);

    return (
        <PageLayout mode="tabs" title="Activity Log" navigation tabs={tabsList}>
            {tabsList.map((tabName) => {
                const isAllTab = tabName === "All";

                const tabEntries = entries.filter(
                    (entry) => isAllTab || entry.pluginName === tabName
                );

                const filteredTabEntries = tabEntries.filter((entry) => {
                    const entryLevel = entry.level.toUpperCase().startsWith("WARN") ? "WARN" : entry.level.toUpperCase();
                    return levelFilter === "ALL" || entryLevel === levelFilter;
                });

                return (
                    <div key={tabName} title={tabName} className="mt-2 w-full h-full">
                        <NoPluginLayout>
                            <section className="flex flex-col h-[calc(100vh-9.5rem)] w-full rounded-md border border-border bg-card shadow-sm overflow-hidden">

                                <div className="shrink-0 p-4 border-b border-border flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between bg-muted/10">
                                    <div className="flex items-center gap-4">
                                        <div className="flex items-center gap-2">
                                            <History className="size-5 text-foreground" />
                                            <h2 className="text-base font-semibold">
                                                {isAllTab ? "All Plugin Activity" : `${tabName} Activity`}
                                            </h2>
                                        </div>

                                        <div className="h-4 w-[1px] bg-border hidden sm:block"></div>

                                        <div className="flex items-center gap-2">
                                            <Filter className="size-3.5 text-muted-foreground" />
                                            <div className="flex bg-muted p-0.5 rounded-md gap-0.5">
                                                {["ALL", "INFO", "WARN", "ERROR"].map((lvl) => (
                                                    <Button
                                                        key={lvl}
                                                        variant="ghost"
                                                        size="sm"
                                                        className={`h-6 px-2.5 text-[10px] cursor-pointer transition-all ${levelFilter === lvl
                                                            ? "bg-background text-foreground shadow-sm hover:bg-background"
                                                            : "text-muted-foreground hover:bg-transparent hover:text-foreground"
                                                            }`}
                                                        onClick={() => setLevelFilter(lvl)}
                                                    >
                                                        {lvl === "WARN" ? "WARNING" : lvl}
                                                    </Button>
                                                ))}
                                            </div>
                                        </div>
                                    </div>

                                    <Button
                                        variant="outline"
                                        size="sm"
                                        className="cursor-pointer gap-2 h-7 text-xs"
                                        disabled={loading}
                                        onClick={loadPluginHistory}
                                    >
                                        <RefreshCw className={`size-3 ${loading ? "animate-spin" : ""}`} />
                                        {loading ? "Loading..." : "Refresh"}
                                    </Button>
                                </div>

                                {error && (
                                    <div className="m-4 shrink-0 rounded-md border border-red-500/40 bg-red-500/10 p-3 text-sm text-red-500">
                                        {error}
                                    </div>
                                )}

                                {!loading && !error && tabEntries.length === 0 && (
                                    <div className="m-4 shrink-0 rounded-md border border-border p-4 text-sm text-muted-foreground text-center">
                                        No Extension history found for {isAllTab ? "any extensions" : tabName}.
                                    </div>
                                )}

                                {!loading && !error && tabEntries.length > 0 && filteredTabEntries.length === 0 && (
                                    <div className="m-4 shrink-0 rounded-md border border-border p-4 text-sm text-muted-foreground text-center">
                                        No logs match the selected level filter.
                                    </div>
                                )}

                                {/* Compact Log List */}
                                {!error && filteredTabEntries.length > 0 && (
                                    <div className="flex-1 overflow-y-auto min-h-0">
                                        <div className="flex flex-col">
                                            {filteredTabEntries.map((entry, index) => (
                                                <LogEntryRow
                                                    key={`${entry.sourceFile}-${entry.timestamp}-${index}`}
                                                    entry={entry}
                                                    isAllTab={isAllTab}
                                                />
                                            ))}
                                        </div>
                                    </div>
                                )}
                            </section>
                        </NoPluginLayout>
                    </div>
                );
            })}
        </PageLayout>
    );
}