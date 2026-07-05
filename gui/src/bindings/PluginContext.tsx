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
    store?: Record<string, any>;
    interactions?: Interaction[];
}

interface PluginContextType {
    plugins: Plugin[];
    currentManifest: PluginManifest | null;
    isManifestLoading: boolean;
    pluginStatus: Record<string, boolean>; // ADDED
    setPluginStatus: React.Dispatch<React.SetStateAction<Record<string, boolean>>>; // ADDED
    refreshPlugins: () => Promise<void>;
    loadPluginManifest: (pluginName: string) => Promise<void>;
    deletePlugin: (pluginName: string) => Promise<void>;
    callPluginFunction: (fnName: string, args: string[]) => Promise<any>;
}
const PluginContext = createContext<PluginContextType | undefined>(undefined);

export function PluginProvider({ children }: { children: ReactNode }) {
    const [plugins, setPlugins] = useState<Plugin[]>([]);
    const [manifests, setManifests] = useState<Record<string, PluginManifest>>({});
    const [isManifestLoading, setIsManifestLoading] = useState(false);
    const [currentManifest, setCurrentManifest] = useState<PluginManifest | null>(null);
    const [pluginStatus, setPluginStatus] = useState<Record<string, boolean>>({});

    async function callPluginFunction(fnName: string, args: string[]): Promise<string> {
        if (!currentManifest?.plugin?.uuid) {
            throw new Error("No plugin manifest loaded");
        }

        const { requestId, promise } = createPendingRequest();

        if (args.length === 0) {
            args = ["{}"];
        }

        await invoke("call_plugin", {
            pluginUuid: currentManifest.plugin.uuid,
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

    async function loadPluginManifest(pluginName: string) {
        if (manifests[pluginName]) {
            setCurrentManifest(manifests[pluginName]);
            return;
        }

        setIsManifestLoading(true);

        try {
            const manifest = await invoke<PluginManifest>("get_plugin_manifest", {
                name: pluginName,
            });

            const normalizedManifest = {
                ...manifest,
                store: manifest.store ?? {},
                interactions: manifest.interactions ?? [],
            };

            setManifests((prev) => ({
                ...prev,
                [pluginName]: normalizedManifest,
            }));

            setCurrentManifest(normalizedManifest);
        } catch (err) {
            error("Failed to load manifest: " + err);
        } finally {
            setIsManifestLoading(false);
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

            setManifests((prev) => {
                const next = { ...prev };
                delete next[pluginName];
                return next;
            });

            setCurrentManifest(null);

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
                currentManifest,
                isManifestLoading,
                pluginStatus,
                setPluginStatus,
                refreshPlugins,
                loadPluginManifest,
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