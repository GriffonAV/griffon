import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import {
    Activity,
    ArrowRight,
    Clock10,
    History,
    RefreshCw,
    Settings2,
    ShieldCheck,
    ToyBrick,
} from "lucide-react";

import { NoPluginLayout } from "@/bindings/component/layout/NoPluginLayout";
import { usePlugins } from "@/bindings/PluginContext";
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
        return "Unknown event";
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

export default function HomePage() {
    const { plugins } = usePlugins();

    const [historyEntries, setHistoryEntries] = useState<PluginHistoryEntry[]>([]);
    const [isLoadingHistory, setIsLoadingHistory] = useState(true);
    const [isRefreshingService, setIsRefreshingService] = useState(false);
    const [historyError, setHistoryError] = useState<string | null>(null);

    const recentHistory = useMemo(
        () => historyEntries.slice(0, 5),
        [historyEntries],
    );

    const latestEvent = recentHistory[0];

    async function loadPluginHistory() {
        try {
            setIsLoadingHistory(true);
            setHistoryError(null);

            const result = await invoke<PluginHistoryEntry[]>("get_plugin_history");
            setHistoryEntries(result);
        } catch (error) {
            console.error("Failed to load plugin history:", error);
            setHistoryError("Unable to load plugin history.");
        } finally {
            setIsLoadingHistory(false);
        }
    }

    async function refreshBackgroundService() {
        try {
            setIsRefreshingService(true);

            await Promise.all([
                invoke("refresh_plugin"),
                loadPluginHistory(),
                new Promise((resolve) => setTimeout(resolve, 500)),
            ]);
        } catch (error) {
            console.error("Failed to refresh Background Service:", error);
        } finally {
            setIsRefreshingService(false);
        }
    }

    useEffect(() => {
        loadPluginHistory();
    }, []);

    return (
        <PageLayout title="Overview">
            <NoPluginLayout>
                <div className="flex w-full flex-col gap-3">
                    <section className="rounded-xl border border-border bg-card p-3 shadow-sm">
                        <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
                            <div>
                                <div className="flex items-center gap-3">
                                    <img
                                        src="/assets/logo.png"
                                        alt="Griffon Logo"
                                        className="h-9 w-9 object-contain"
                                        style={{ imageRendering: "pixelated" }}
                                    />

                                    <div>
                                        <h2 className="text-2xl font-bold">Griffon Dashboard</h2>

                                        <p className="mt-1 text-sm text-muted-foreground">
                                            Overview of installed plugins, recent activity and system controls.
                                        </p>
                                    </div>
                                </div>
                            </div>

                            <Button
                                variant="outline"
                                className="w-fit cursor-pointer gap-2"
                                disabled={isRefreshingService}
                                onClick={refreshBackgroundService}
                            >
                                <RefreshCw
                                    className={`size-4 ${isRefreshingService ? "animate-spin" : ""}`}
                                />

                                {isRefreshingService ? "Refreshing..." : "Refresh service"}
                            </Button>
                        </div>
                    </section>

                    <section className="grid gap-4 md:grid-cols-3">
                        <div className="rounded-xl border border-border bg-card p-5 shadow-sm">
                            <div className="flex items-center justify-between">
                                <p className="text-sm font-medium text-muted-foreground">
                                    Installed plugins
                                </p>

                                <ToyBrick className="size-5 text-muted-foreground" />
                            </div>

                            <p className="mt-4 text-3xl font-bold">{plugins.length}</p>

                            <p className="mt-1 text-sm text-muted-foreground">
                                Plugins detected by Griffon.
                            </p>
                        </div>

                        <div className="rounded-xl border border-border bg-card p-5 shadow-sm">
                            <div className="flex items-center justify-between">
                                <p className="text-sm font-medium text-muted-foreground">
                                    History events
                                </p>

                                <History className="size-5 text-muted-foreground" />
                            </div>

                            <p className="mt-4 text-3xl font-bold">{historyEntries.length}</p>

                            <p className="mt-1 text-sm text-muted-foreground">
                                Events found in plugin history files.
                            </p>
                        </div>

                        <div className="rounded-xl border border-border bg-card p-5 shadow-sm">
                            <div className="flex items-center justify-between">
                                <p className="text-sm font-medium text-muted-foreground">
                                    Latest event
                                </p>

                                <Clock10 className="size-5 text-muted-foreground" />
                            </div>

                            <p className="mt-4 truncate text-lg font-bold">
                                {latestEvent ? formatEvent(latestEvent.event) : "No activity"}
                            </p>

                            <p className="mt-1 truncate text-sm text-muted-foreground">
                                {latestEvent
                                    ? `${latestEvent.pluginName} • ${formatTimestamp(latestEvent.timestamp)}`
                                    : "No plugin history available yet."}
                            </p>
                        </div>
                    </section>

                    <section className="grid gap-3 xl:grid-cols-[1.1fr_0.9fr]">
                        <div className="rounded-xl border border-border bg-card p-3 shadow-sm">
                            <div className="flex items-center justify-between gap-4">
                                <div>
                                    <h3 className="text-xl font-bold">Recent plugin activity</h3>

                                    <p className="mt-1 text-sm text-muted-foreground">
                                        Last events from the plugin history folder.
                                    </p>
                                </div>

                                <Link to="/log">
                                    <Button variant="ghost" className="cursor-pointer gap-2">
                                        Open logs
                                        <ArrowRight className="size-4" />
                                    </Button>
                                </Link>
                            </div>

                            <div className="mt-5 flex flex-col gap-3">
                                {isLoadingHistory && (
                                    <p className="text-sm text-muted-foreground">
                                        Loading plugin history...
                                    </p>
                                )}

                                {historyError && (
                                    <div className="rounded-md border border-red-500/40 bg-red-500/10 p-4 text-sm text-red-500">
                                        {historyError}
                                    </div>
                                )}

                                {!isLoadingHistory && !historyError && recentHistory.length === 0 && (
                                    <div className="rounded-md border border-border p-4 text-sm text-muted-foreground">
                                        No plugin activity found.
                                    </div>
                                )}

                                {!historyError &&
                                    recentHistory.map((entry, index) => (
                                        <div
                                            key={`${entry.sourceFile}-${entry.timestamp}-${index}`}
                                            className="rounded-md border border-border p-4"
                                        >
                                            <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                                                <div className="min-w-0">
                                                    <div className="flex flex-wrap items-center gap-2">
                                                        <p className="font-semibold">{entry.pluginName}</p>

                                                        <span
                                                            className={`text-xs font-medium ${getLevelClass(
                                                                entry.level,
                                                            )}`}
                                                        >
                                                            ● {entry.level}
                                                        </span>
                                                    </div>

                                                    <p className="mt-1 text-sm text-muted-foreground">
                                                        {formatEvent(entry.event)}
                                                    </p>

                                                    {entry.path && (
                                                        <p className="mt-2 truncate text-xs text-muted-foreground">
                                                            {entry.path}
                                                        </p>
                                                    )}
                                                </div>

                                                <p className="shrink-0 text-xs text-muted-foreground">
                                                    {formatTimestamp(entry.timestamp)}
                                                </p>
                                            </div>
                                        </div>
                                    ))}
                            </div>
                        </div>

                        <div className="flex flex-col gap-3">
                            <div className="rounded-xl border border-border bg-card p-3 shadow-sm">
                                <h3 className="text-xl font-bold">Quick actions</h3>

                                <p className="mt-1 text-sm text-muted-foreground">
                                    Common Griffon actions.
                                </p>

                                <div className="mt-5 grid gap-3">
                                    <Link to="/log">
                                        <Button
                                            variant="outline"
                                            className="w-full cursor-pointer justify-start gap-3"
                                        >
                                            <Activity className="size-4" />
                                            View activity log
                                        </Button>
                                    </Link>

                                    <Link to="/settings">
                                        <Button
                                            variant="outline"
                                            className="w-full cursor-pointer justify-start gap-3"
                                        >
                                            <Settings2 className="size-4" />
                                            Manage settings
                                        </Button>
                                    </Link>

                                    <Button
                                        variant="outline"
                                        className="w-full cursor-pointer justify-start gap-3"
                                        disabled={isRefreshingService}
                                        onClick={refreshBackgroundService}
                                    >
                                        <RefreshCw
                                            className={`size-4 ${isRefreshingService ? "animate-spin" : ""
                                                }`}
                                        />
                                        Refresh background service
                                    </Button>
                                </div>
                            </div>

                            <div className="rounded-xl border border-border bg-card p-3 shadow-sm">
                                <div className="flex items-center justify-between gap-4">
                                    <div>
                                        <h3 className="text-xl font-bold">Installed plugins</h3>

                                        <p className="mt-1 text-sm text-muted-foreground">
                                            Quick access to plugin pages.
                                        </p>
                                    </div>

                                    <ToyBrick className="size-5 text-muted-foreground" />
                                </div>

                                <div className="mt-5 flex flex-col gap-3">
                                    {plugins.length === 0 && (
                                        <div className="rounded-md border border-border p-4 text-sm text-muted-foreground">
                                            No plugin available.
                                        </div>
                                    )}

                                    {plugins.slice(0, 5).map((plugin) => (
                                        <Link
                                            key={plugin.uuid}
                                            to={`/plugin/${plugin.file_name}`}
                                            className="rounded-md border border-border p-3 transition hover:bg-muted"
                                        >
                                            <div className="flex items-center justify-between gap-3">
                                                <div className="min-w-0">
                                                    <p className="truncate font-semibold">
                                                        {plugin.display_name}
                                                    </p>

                                                    <p className="mt-1 truncate text-xs text-muted-foreground">
                                                        {plugin.uuid}
                                                    </p>
                                                </div>

                                                <ArrowRight className="size-4 shrink-0 text-muted-foreground" />
                                            </div>
                                        </Link>
                                    ))}
                                </div>
                            </div>
                        </div>
                    </section>
                </div>
            </NoPluginLayout>
        </PageLayout>
    );
}