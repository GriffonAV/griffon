import { createContext, useContext, useEffect, useState, ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface Plugin {
    pid: number;
    name: string;
}

interface PluginContextType {
    plugins: Plugin[];
    isLoading: boolean;
    refreshPlugins: () => void;
}

const PluginContext = createContext<PluginContextType | undefined>(undefined);

export function PluginProvider({ children }: { children: ReactNode }) {
    const [plugins, setPlugins] = useState<Plugin[]>([]);
    const [isLoading, setIsLoading] = useState(true);

    async function refreshPlugins() {
        setIsLoading(true);
        try {
            const result = await invoke<Plugin[]>("list_plugins_cmd");
            setPlugins(result);
        } catch (err) {
            console.error("Failed to load plugins:", err);
        } finally {
            setIsLoading(false);
        }
    }

    useEffect(() => {
        refreshPlugins();
    }, []);

    return (
        <PluginContext.Provider value={{ plugins, isLoading, refreshPlugins }}>
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
