import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import {
    ArrowRight,
    Clock10,
    RefreshCw,
    Settings2,
    Sparkles,
} from "lucide-react";

import { NoPluginLayout } from "@/bindings/component/layout/NoPluginLayout";
import { usePlugins } from "@/bindings/PluginContext";
import { PageLayout } from "@/components/layout/PageLayout";
import { Button } from "@/components/ui/button";
import { getVersion } from '@tauri-apps/api/app';

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
    if (!event) return "Unknown event";
    return event.replaceAll("_", " ");
}

export default function HomePage() {
  const { plugins } = usePlugins();


    const [historyEntries, setHistoryEntries] = useState<PluginHistoryEntry[]>([]);
    const [isRefreshingService, setIsRefreshingService] = useState(false);

    const latestEvent = historyEntries[0];

    const [version, setVersion] = useState("");

      useEffect(() => {
        getVersion().then(setVersion);
      }, []);

    async function loadPluginHistory() {
        try {
            const result = await invoke<PluginHistoryEntry[]>("get_plugin_history");
            setHistoryEntries(result);
        } catch (error) {
            console.error("Failed to load plugin history:", error);
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
                <div className="flex w-full flex-col gap-6">
                    <div className="flex items-center gap-3 rounded-md border border-primary/20 bg-primary/10 p-3 text-sm text-primary shadow-sm">
                        <Sparkles className="size-4 shrink-0" />
                        <p>
                            <strong className="font-semibold">Griffon {version} is out!</strong> Enjoy the new minimalist dashboard, global notification controls, and improved plugin history.
                        </p>
                    </div>
                    <div className="flex items-center justify-between border-b border-border pb-4">
                        <div>
                            <h2 className="text-xl font-bold tracking-tight">Dashboard</h2>
                            <p className="text-sm text-muted-foreground">System overview and quick access.</p>
                        </div>
                        <Button
                            variant="outline"
                            size="sm"
                            className="w-fit cursor-pointer gap-2"
                            disabled={isRefreshingService}
                            onClick={refreshBackgroundService}
                        >
                            <RefreshCw className={`size-4 ${isRefreshingService ? "animate-spin" : ""}`} />
                            {isRefreshingService ? "Refreshing..." : "Refresh daemon"}
                        </Button>
                    </div>

                    <div className="grid gap-4 md:grid-cols-3">
                        <div className="rounded-md border border-border bg-card p-4">
                            <p className="text-sm font-medium text-muted-foreground">Installed Extensions</p>
                            <p className="mt-2 text-2xl font-bold">{plugins.length}</p>
                        </div>
                        <div className="rounded-md border border-border bg-card p-4">
                            <p className="text-sm font-medium text-muted-foreground">Activity Entries</p>
                            <p className="mt-2 text-2xl font-bold">{historyEntries.length}</p>
                        </div>
                        <div className="rounded-md border border-border bg-card p-4">
                            <p className="text-sm font-medium text-muted-foreground">Latest activity</p>
                            <div className="mt-2">
                                <p className="truncate text-base font-bold">
                                    {latestEvent ? formatEvent(latestEvent.event) : "No activity"}
                                </p>
                                <p className="truncate text-xs text-muted-foreground mt-0.5">
                                    {latestEvent ? `${latestEvent.pluginName} • ${formatTimestamp(latestEvent.timestamp)}` : "-"}
                                </p>
                            </div>
                        </div>
                    </div>

                    <div className="grid gap-6 xl:grid-cols-[1fr_250px]">

                        <div className="flex flex-col gap-3">
                            <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">
                                Installed Extensions
                            </h3>

                            {plugins.length === 0 ? (
                                <p className="text-sm text-muted-foreground border border-border rounded-md p-4 bg-muted/20">
                                    No extensions currently installed.
                                </p>
                            ) : (
                                <div className="flex flex-col gap-2">
                                    {plugins.map((plugin) => (
                                        <Link
                                            key={plugin.uuid}
                                            to={`/plugin/${plugin.file_name}`}
                                            className="group flex items-center justify-between rounded-md border border-border bg-card p-3 transition-colors hover:bg-muted/50"
                                        >
                                            <div>
                                                <p className="text-sm font-semibold">{plugin.display_name}</p>
                                                <p className="text-xs text-muted-foreground mt-0.5">
                                                    v{plugin.version} • {plugin.file_name}
                                                </p>
                                            </div>
                                            <ArrowRight className="size-4 text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100" />
                                        </Link>
                                    ))}
                                </div>
                            )}
                        </div>

                        <div className="flex flex-col gap-3">
                            <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">
                                Shortcuts
                            </h3>
                            <div className="flex flex-col gap-2">
                                <Button asChild variant="secondary" className="w-full justify-start gap-3 cursor-pointer">
                                    <Link to="/log">
                                        <Clock10 className="size-4 text-muted-foreground" />
                                        Activity Log
                                    </Link>
                                </Button>
                                <Button asChild variant="secondary" className="w-full justify-start gap-3 cursor-pointer">
                                    <Link to="/settings">
                                        <Settings2 className="size-4 text-muted-foreground" />
                                        Settings
                                    </Link>
                                </Button>
                            </div>
                        </div>

                    </div>
                </div>
            </NoPluginLayout>
        </PageLayout>
    );
}
