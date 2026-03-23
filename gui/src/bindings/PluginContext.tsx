import { createContext, useContext, useEffect, useState, ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
    debug,
    error
} from '@tauri-apps/plugin-log';

export interface Plugin {
    pid: number;
    name: string;
}

export interface PluginManifest {
    plugin: {
        name: string;
        id: string;
        version: string;
        author: string;
        description: string;
        tabs: string[];
    };
    ui: {
        sections: Array<{
            id: string;
            tab: string;
        }>;
    };
}

interface PluginContextType {
    plugins: Plugin[];
    isLoading: boolean;
    currentManifest: PluginManifest | null;
    isManifestLoading: boolean;
    refreshPlugins: () => void;
    loadPluginManifest: (pluginId: string) => void;
}


const PluginContext = createContext<PluginContextType | undefined>(undefined);

export function PluginProvider({ children }: { children: ReactNode }) {
    const [plugins, setPlugins] = useState<Plugin[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [currentManifest, setCurrentManifest] = useState<PluginManifest | null>(null);
    const [isManifestLoading, setIsManifestLoading] = useState(false);

    async function refreshPlugins() {
        setIsLoading(true);
        try {
            const result = await invoke<Plugin[]>("list_plugins_cmd");
            setPlugins(result);
        } catch (err) {
            error("Failed to load plugins:" + err);
        } finally {
            setIsLoading(false);
        }
    }

    async function loadPluginManifest(pluginId: string) {
        setIsManifestLoading(true);
        try {
            const manifest = await invoke<PluginManifest>("get_plugin_manifest", { pluginId });
            debug("Loaded manifest:" + JSON.stringify(manifest));
            setCurrentManifest(manifest);
        } catch (err) {
            error("Failed to load manifest:" + err);
        } finally {
            setIsManifestLoading(false);
        }
    }

    useEffect(() => {
        refreshPlugins();
    }, []);

    return (
        <PluginContext.Provider
            value={{
                plugins,
                isLoading,
                currentManifest,
                isManifestLoading,
                refreshPlugins,
                loadPluginManifest,
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
