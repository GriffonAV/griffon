import { createContext, useContext, useEffect, useState } from "react";
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
    pluginStatus: Record<string, boolean>; // ADDED
    setPluginStatus: React.Dispatch<React.SetStateAction<Record<string, boolean>>>; // ADDED
    refreshPlugins: () => Promise<void>;
    loadPlugin: (pluginName: string) => Promise<void>;
    deletePlugin: (pluginName: string) => Promise<void>;
    callPluginFunction: (fnName: string, args: string[]) => Promise<any>;
}
const PluginContext = createContext<PluginContextType | undefined>(undefined);

export function PluginProvider({ children }: { children: ReactNode }) {
    const [plugins, setPlugins] = useState<Plugin[]>([]);
    const [isPluginLoading, setIsPluginLoading] = useState(false);
    const [currentPlugin, setCurrentPlugin] = useState<Plugin | null>(null);
    const [pluginStatus, setPluginStatus] = useState<Record<string, boolean>>({});

    async function callPluginFunction(fnName: string, args: string[]): Promise<string> {
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
    }

    async function refreshPlugins() {
        try {
            const result = await invoke<Plugin[]>("list_plugins");
            setPlugins(result);
        } catch (err) {
            error("Failed to load plugins: " + err);
        }
    }

    async function loadPlugin(pluginName: string) {
        setIsPluginLoading(true);

        const plugin = plugins.find((p) => p.file_name === pluginName);
        
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
                uuid: plugin.uuid,
            });

            const normalizedManifest : PluginManifest = {
                ...manifest,
                store: manifest.store ?? {},
                interactions: manifest.interactions ?? [],
            };

            const updatedPlugin = { ...plugin, manifest: normalizedManifest };
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

    async function deletePlugin(pluginName: string) {
        try {
            await invoke("delete_plugin", {
                name: pluginName,
            });

            setPlugins((prev) =>
                prev.filter((plugin) => plugin.file_name !== pluginName)
            );

            if (currentPlugin?.file_name === pluginName) {
                setCurrentPlugin(null);
            }

            await refreshPlugins();
        } catch (err) {
            error("Failed to delete plugin: " + err);
            throw err;
        }
    }

    useEffect(() => {
        refreshPlugins();
    }, []);

    return (
        <PluginContext.Provider
            value={{
                plugins,
                currentPlugin,
                isPluginLoading,
                pluginStatus,
                setPluginStatus,
                refreshPlugins,
                loadPlugin,
                deletePlugin,
                callPluginFunction,
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