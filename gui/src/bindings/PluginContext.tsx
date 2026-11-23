import { createContext, useContext, useEffect, useState } from "react";
import type { ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { error } from "@tauri-apps/plugin-log";
import { listen } from "@tauri-apps/api/event";
import { createPendingRequest, initializeRequestManager, type DaemonPlugin } from "@/services/requestManager";
import { debugLog } from "@/services/debugManager";
import { uuidBytesToString } from "@/lib/utils";

export type Plugin = {
    file_name: string;
    uuid: string;
    display_name: string;
    version: string;
    author: string;
    description: string;
    notifications_enabled: boolean;
    notification_status: "idle" | "busy" | "error";
    manifest: PluginManifest | null;
    status: "healthy" | "busy" | "disabled" | "error";
};

export interface InteractionStep {
    type: string;
    key?: string;
    value?: any;
    from?: string;
    amount?: number;
    [key: string]: any;
}

export interface Interaction {
    id: string;
    on: string;
    steps: InteractionStep[];
}

export interface PluginManifest {
    plugin: {
        name: string;
        id: string;
        version: string;
        author: string;
        description: string;
        tabs: string[];
        uuid: string;
    };
    ui: {
        sections: Array<{
            id: string;
            tab: string;
            contents: Array<{
                type: string;
                id: string;
                [key: string]: any;
            }>;
        }>;
    };
    store: Record<string, any>;
    interactions: Interaction[];
}

interface PluginContextType {
    plugins: Plugin[];
    currentPlugin: Plugin | null;
    isPluginLoading: boolean;
    refreshPlugins: () => Promise<void>;
    loadPlugin: (pluginName: string) => Promise<void>;
    deletePlugin: (pluginName: string) => Promise<void>;
    callPluginFunction: (fnName: string, args: string[]) => Promise<any>;
    switchPluginStatus: (pluginUuid: string) => Promise<void>;
    switchPluginNotificationStatus: (pluginUuid: string) => Promise<void>;
}
const PluginContext = createContext<PluginContextType | undefined>(undefined);

export function PluginProvider({ children }: { children: ReactNode }) {
    const [plugins, setPlugins] = useState<Plugin[]>([]);
    const [isPluginLoading, setIsPluginLoading] = useState(false);
    const [currentPlugin, setCurrentPlugin] = useState<Plugin | null>(null);

    async function callPluginFunction(fnName: string, args: string[]): Promise<string> {
        if (!currentPlugin) {
            throw new Error("No plugin loaded");
        }

        if (currentPlugin.status !== "healthy") {
            throw new Error(`Plugin ${currentPlugin.display_name} is not enabled`);
        }

        await initializeRequestManager();
        const { requestId, promise } = createPendingRequest();

        if (args.length === 0) {
            args = ["{}"];
        }

        await invoke("call_plugin", {
            pluginUuid: currentPlugin.uuid,
            fnName,
            args,
            requestId,
        });

        const result = await promise;
        return result;
    }

    async function switchPluginStatus(pluginUuid: string): Promise<void> {
        await initializeRequestManager();
        const { requestId, promise } = createPendingRequest();

        setPlugins((prev) =>
            prev.map((plugin) =>
                plugin.uuid === pluginUuid
                    ? { ...plugin, status: "busy" }
                    : plugin
            )
        );

        try {
            await invoke("switch_status_plugin", {
                pluginUuid,
                requestId,
            });

            const result = await promise;
            setPlugins((prev) =>
                prev.map((plugin) =>
                    plugin.uuid === pluginUuid
                        ? { ...plugin,
                            status: result.enable ? "healthy" : "disabled",
                        } : plugin
                )
            );
            debugLog("Plugin status switched successfully:", result);
        } catch (err) {
            error(`Failed to switch plugin status: ${err}`);
            await refreshPlugins();
        }
    }

    async function switchPluginNotificationStatus(pluginUuid: string): Promise<void> {
        await initializeRequestManager();
        const { requestId, promise } = createPendingRequest<{ enable: boolean }>();

        setPlugins((previous) =>
            previous.map((plugin) =>
                plugin.uuid === pluginUuid
                    ? { ...plugin, notification_status: "busy" }
                    : plugin
            )
        );

        try {
            await invoke("switch_status_notification", {
                pluginUuid,
                requestId,
            });

            const result = await promise;

            setPlugins((previous) =>
                previous.map((plugin) =>
                    plugin.uuid === pluginUuid
                        ? {
                            ...plugin,
                            notifications_enabled: result.enable,
                            notification_status: "idle",
                        }
                        : plugin
                )
            );
        } catch (err) {
            setPlugins((previous) =>
                previous.map((plugin) =>
                    plugin.uuid === pluginUuid
                        ? { ...plugin, notification_status: "error" }
                        : plugin
                )
            );
            error(`Failed to switch plugin notification status: ${err}`);
            throw err;
        }
    }

    async function refreshPlugins() {
        debugLog("Refreshing plugins...");

        let pluginList: Array<Omit<Plugin, "manifest" | "status" | "notification_status">>;
        try {
            pluginList = await invoke("list_plugins");
        } catch (err) {
            error(`Failed to load plugins: ${err}`);
            return;
        }

        setPlugins((previous) =>
            pluginList.map((plugin) => {
                const existing = previous.find((item) => item.uuid === plugin.uuid);

                return {
                    ...plugin,
                    manifest: existing?.manifest ?? null,
                    // Unknown until the daemon confirms it below.
                    status: existing?.status ?? "busy",
                    notification_status: "idle",
                };
            })
        );

        try {
            // The daemon reply arrives as a Tauri event: make sure the
            // listeners exist before sending the request.
            await initializeRequestManager();
            const { requestId, promise } = createPendingRequest<DaemonPlugin[]>();
            await invoke("refresh_plugin", { requestId });
            const result = await promise;

            // The daemon reports `status: false` both for plugins the user
            // disabled and for failed enable attempts; both can be re-enabled
            // from Settings, so show them as "disabled". Plugins the daemon
            // does not report (or reports without a UUID after a failed
            // handshake) are not usable: show them as "error".
            const daemonStatuses = new Map<string, Plugin["status"]>(
                result.map((plugin) => [
                    uuidBytesToString(plugin.plugin_uuid),
                    plugin.status ? "healthy" : "disabled",
                ])
            );

            setPlugins((prev) =>
                prev.map((p) => ({
                    ...p,
                    status: daemonStatuses.get(p.uuid.toLowerCase()) ?? "error",
                }))
            );
            debugLog("Plugin statuses refreshed:", result);
        } catch (err) {
            error(`Failed to refresh plugin statuses: ${err}`);
            // Without a daemon answer no plugin state is confirmed.
            setPlugins((prev) => prev.map((p) => ({ ...p, status: "error" })));
        }
    }

    async function loadPlugin(pluginName: string) {
        setIsPluginLoading(true);

        const plugin = plugins.find((p) => p.file_name === pluginName);
        
        if (!plugin) { // Check if the plugin exists in the list
            error(`Plugin ${pluginName} not found`);
            setCurrentPlugin(null);
            setIsPluginLoading(false);
            return;
        }

        if (plugin.status !== "healthy") {
            error(`Plugin ${pluginName} is not enabled`);
            setCurrentPlugin(null);
            setIsPluginLoading(false);
            return;
        }

        if (plugin?.manifest) { // Check if the manifest is already loaded
            setCurrentPlugin(plugin);
            setIsPluginLoading(false);
            return;
        }

        try { // Try to load the manifest
            const manifest = await invoke<PluginManifest>("get_plugin_manifest", {
                name: plugin.file_name,
            });

            const normalizedManifest : PluginManifest = {
                ...manifest,
                store: manifest.store ?? {},
                interactions: manifest.interactions ?? [],
            };

            const updatedPlugin = { ...plugin, manifest: normalizedManifest } satisfies Plugin;
            setCurrentPlugin(updatedPlugin);
            setPlugins((prev) =>
                prev.map((p) => (p.file_name === pluginName ? updatedPlugin : p))
            );
        } catch (err) {
            error(`Failed to load plugin manifest for\nname: ${pluginName}\nuuid: ${plugin.uuid}\nerror: ${err}`);
        } finally {
            setIsPluginLoading(false);
        }
    }

    async function deletePlugin(pluginUuid: string) {
        try {
            setPlugins((prev) =>
                prev.map((plugin) =>
                    plugin.uuid === pluginUuid
                        ? { ...plugin, status: "busy" }
                        : plugin
                )
            );

            await invoke("delete_plugin", {
                pluginUuid,
            });

            setPlugins((prev) =>
                prev.filter((plugin) => plugin.uuid !== pluginUuid)
            );

            if (currentPlugin?.uuid === pluginUuid) {
                setCurrentPlugin(null);
            }

            await refreshPlugins();
        } catch (err) {
            error("Failed to delete plugin: " + err);
            throw err;
        }
    }

    useEffect(() => {
        setCurrentPlugin((prev) => {
            if (prev) {
                const updatedPlugin = plugins.find((p) => p.uuid === prev.uuid);
                return updatedPlugin ?? null;
            }
            return null;
        });
    }, [plugins]);

    useEffect(() => {
        debugLog("Initializing PluginProvider and refreshing plugins...");
        void refreshPlugins();

        // Statuses fetched while the daemon was unreachable are "error":
        // refresh them once the GUI (re)connects.
        let disposed = false;
        let unlisten: (() => void) | undefined;
        listen<{ status: string } | string>("daemon-status", ({ payload }) => {
            const status = typeof payload === "string" ? payload : payload?.status;
            if (status === "Connected") {
                void refreshPlugins();
            }
        })
            .then((fn) => {
                if (disposed) {
                    fn();
                } else {
                    unlisten = fn;
                }
            })
            .catch((err) => error(`Failed to listen for daemon status: ${err}`));

        return () => {
            disposed = true;
            unlisten?.();
        };
    }, []);

    return (
        <PluginContext.Provider
            value={{
                plugins,
                currentPlugin,
                isPluginLoading,
                refreshPlugins,
                loadPlugin,
                deletePlugin,
                callPluginFunction,
                switchPluginStatus,
                switchPluginNotificationStatus,
            }}
        >
            {children}
        </PluginContext.Provider>
    );
}

export function usePlugins() {
    const context = useContext(PluginContext);

    if (context === undefined) {
        throw new Error("usePlugins must be used within a PluginProvider");
    }

    return context;
}
