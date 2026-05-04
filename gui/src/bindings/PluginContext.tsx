import { createContext, useContext, useEffect, useState } from "react";
import type { ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { error } from "@tauri-apps/plugin-log";
import { listen } from "@tauri-apps/api/event";

export interface Plugin {
    pid: number;
    name: string;
}

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
    refreshPlugins: () => void;
    loadPluginManifest: (pluginId: string) => void;
    callPluginFunction: (fnName: string, args: string[]) => Promise<any>;
}

interface PluginCallResult {
    request_id: number;
    ok: boolean;
    output: any;
}

const PluginContext = createContext<PluginContextType | undefined>(undefined);

export function PluginProvider({ children }: { children: ReactNode }) {
    const [plugins, setPlugins] = useState<Plugin[]>([]);

    const [manifests, setManifests] = useState<Record<string, PluginManifest>>({});
    const [isManifestLoading, setIsManifestLoading] = useState(false);
    const [currentManifest, setCurrentManifest] = useState<PluginManifest>({} as PluginManifest);

    const pending = new Map<number, { resolve: Function; reject: Function }>();
    
    let requestCounter = 0;

    listen("plugin-call-result", (event) => {
        const { request_id, ok, output } = event.payload as PluginCallResult;

        const entry = pending.get(request_id);
        if (!entry) return;

        pending.delete(request_id);

        if (ok) entry.resolve(output);
        else entry.reject(new Error(output));
    });

    async function callPluginFunction(fnName: string, args: string[]) {
        requestCounter += 1;
        const requestId : number = requestCounter;

        return new Promise(async (resolve, reject) => {
            pending.set(requestId, { resolve, reject });
    
            await invoke("call_plugin", {
                pluginUuid: currentManifest.plugin.uuid,
                fnName,
                args,
                requestId,
            });
        });
    }

    async function refreshPlugins() {
        try {
            const result = await invoke<Plugin[]>("list_plugins");
            setPlugins(result);
        } catch (err) {
            error("Failed to load plugins:" + err);
        }
    }

    async function loadPluginManifest(pluginName: string) {

        if (manifests[pluginName]) {
            setCurrentManifest(manifests[pluginName]);
            return;
        }

        setIsManifestLoading(true);

        try {
            const manifest = await invoke<PluginManifest>("get_plugin_manifest", { "name": pluginName });

            setManifests(prev => ({
            ...prev,
            [pluginName]: {
                ...manifest,
                store: manifest.store ?? {},
                interactions: manifest.interactions ?? [],
            }
            }));

            setCurrentManifest(manifests[pluginName]);
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
                currentManifest,
                isManifestLoading,
                refreshPlugins,
                loadPluginManifest,
                callPluginFunction
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


