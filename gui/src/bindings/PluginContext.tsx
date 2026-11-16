import {
    createContext,
    useCallback,
    useContext,
    useEffect,
    useMemo,
    useRef,
    useState,
} from "react";
import type { ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { error } from "@tauri-apps/plugin-log";
import { createPendingRequest } from "@/services/requestManager";

export type Plugin = {
    file_name: string;
    uuid: string;
    display_name: string;
    version: string;
    author: string;
    description: string;
    notifications_enabled: boolean;
    manifest: PluginManifest | null;
    status: "healthy" | "busy" | "disabled" | "no_manifest" | "error";
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
            title?: string;
            description?: string;
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
}
const PluginContext = createContext<PluginContextType | undefined>(undefined);

function normalizePlugins(plugins: Plugin[]): Plugin[] {
    return plugins.map((plugin) => ({
        ...plugin,
        status: plugin.manifest ? "healthy" as const : "no_manifest" as const,
    }));
}

export function PluginProvider({ children }: { children: ReactNode }) {
    const [plugins, setPlugins] = useState<Plugin[]>([]);
    const [isPluginLoading, setIsPluginLoading] = useState(false);
    const [currentPlugin, setCurrentPlugin] = useState<Plugin | null>(null);
    const pluginsRef = useRef(plugins);

    useEffect(() => {
        pluginsRef.current = plugins;
    }, [plugins]);

    const refreshPlugins = useCallback(async () => {
        try {
            const result = await invoke<Plugin[]>("list_plugins");
            const normalized = normalizePlugins(result);
            pluginsRef.current = normalized;
            setPlugins(normalized);
        } catch (err) {
            error("Failed to load plugins: " + err);
        }
    }, []);

    const callPluginFunction = useCallback(async (fnName: string, args: string[]): Promise<string> => {
        if (!currentPlugin) {
            throw new Error("No plugin loaded");
        }

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
    }, [currentPlugin]);

    const switchPluginStatus = useCallback(async (pluginUuid: string): Promise<void> => {
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
        } catch (err) {
            error(`Failed to switch plugin status: ${err}`);
            await refreshPlugins();
        }
    }, [refreshPlugins]);

    const loadPlugin = useCallback(async (pluginName: string) => {
        setIsPluginLoading(true);

        let plugin = pluginsRef.current.find((p) => p.file_name === pluginName);

        if (!plugin) {
            try {
                const result = await invoke<Plugin[]>("list_plugins");
                const normalized = normalizePlugins(result);
                pluginsRef.current = normalized;
                setPlugins(normalized);
                plugin = normalized.find((candidate) => candidate.file_name === pluginName);
            } catch (err) {
                error(`Failed to refresh plugins before loading ${pluginName}: ${err}`);
            }
        }

        if (!plugin) { // Check if the plugin exists in the list
            error(`Plugin ${pluginName} not found`);
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

            const updatedPlugin = { ...plugin, manifest: normalizedManifest, status: "healthy" } satisfies Plugin;
            setCurrentPlugin(updatedPlugin);
            setPlugins((prev) => {
                const next = prev.map((p) => (p.file_name === pluginName ? updatedPlugin : p));
                pluginsRef.current = next;
                return next;
            });
        } catch (err) {
            error(`Failed to load plugin manifest for\nname: ${pluginName}\nuuid: ${plugin.uuid}\nerror: ${err}`);
        } finally {
            setIsPluginLoading(false);
        }
    }, []);

    const deletePlugin = useCallback(async (pluginUuid: string) => {
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
    }, [currentPlugin, refreshPlugins]);

    useEffect(() => {
        refreshPlugins();
    }, [refreshPlugins]);

    const value = useMemo(() => ({
        plugins,
        currentPlugin,
        isPluginLoading,
        refreshPlugins,
        loadPlugin,
        deletePlugin,
        callPluginFunction,
        switchPluginStatus,
    }), [
        callPluginFunction,
        currentPlugin,
        deletePlugin,
        isPluginLoading,
        loadPlugin,
        plugins,
        refreshPlugins,
        switchPluginStatus,
    ]);

    return (
        <PluginContext.Provider value={value}>
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
