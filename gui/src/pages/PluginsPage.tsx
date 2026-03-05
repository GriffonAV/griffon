import React from "react";
import { usePlugins } from "../hooks/usePlugins";

export default function PluginsPage() {
    const { plugins, isLoading, refreshPlugins } = usePlugins();

    if (isLoading) {
        return (
            <div style={{ display: "flex", justifyContent: "center", padding: "2rem" }}>
                <p>Loading plugins...</p>
            </div>
        );
    }

    if (plugins.length === 0) {
        return (
            <div style={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", height: "100%", padding: "2rem", textAlign: "center" }}>
                <h2>No Plugins Loaded</h2>
                <p style={{ marginBottom: "1.5rem", color: "#666" }}>
                    The daemon is running, but no plugins were found.
                </p>
                <button onClick={refreshPlugins} style={{ padding: "0.5rem 1rem", cursor: "pointer" }}>
                    Refresh
                </button>
            </div>
        );
    }

    return (
        <div style={{ padding: "2rem" }}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "1rem" }}>
                <h2>Loaded Plugins</h2>
                <button onClick={refreshPlugins} style={{ padding: "0.25rem 0.5rem", cursor: "pointer" }}>Refresh</button>
            </div>
            <ul style={{ listStyle: "none", padding: 0 }}>
                {plugins.map((plugin) => (
                    <li key={plugin.pid} style={{ padding: "1rem", border: "1px solid #ddd", marginBottom: "0.5rem", borderRadius: "4px" }}>
                        <strong>{plugin.name}</strong> <span style={{ color: "#888", fontSize: "0.9em" }}>(PID: {plugin.pid})</span>
                    </li>
                ))}
            </ul>
        </div>
    );
}
