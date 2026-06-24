import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

type InstalledPlugin = {
    toml_path: string;
    so_path: string;
    plugin_dir: string;
};

export function PluginInstaller() {
    const [tomlPath, setTomlPath] = useState<string | null>(null);
    const [soPath, setSoPath] = useState<string | null>(null);
    const [status, setStatus] = useState<string | null>(null);
    const [isInstalling, setIsInstalling] = useState(false);

    const pickToml = async () => {
        const selected = await open({
            multiple: false,
            filters: [
                {
                    name: "Griffon plugin manifest",
                    extensions: ["toml"],
                },
            ],
        });

        if (typeof selected === "string") {
            setTomlPath(selected);
            setStatus(null);
        }
    };

    const pickSo = async () => {
        const selected = await open({
            multiple: false,
            filters: [
                {
                    name: "Griffon plugin library",
                    extensions: ["so"],
                },
            ],
        });

        if (typeof selected === "string") {
            setSoPath(selected);
            setStatus(null);
        }
    };

    const installPlugin = async () => {
        if (!tomlPath || !soPath) {
            setStatus("Please select both a .toml file and a .so file.");
            return;
        }

        try {
            setIsInstalling(true);
            setStatus("Installing plugin...");

            const result = await invoke<InstalledPlugin>("install_plugin_files", {
                tomlPath,
                soPath,
            });

            setStatus(`Plugin installed successfully in ${result.plugin_dir}`);
        } catch (error) {
            setStatus(`Installation failed: ${String(error)}`);
        } finally {
            setIsInstalling(false);
        }
    };

    return (
        <div className="rounded-lg border border-border p-4 space-y-4">
            <div>
                <h2 className="text-xl font-bold">Install plugin</h2>
                <p className="text-sm text-muted-foreground">
                    Select the plugin manifest and shared library. They will be copied to .config/griffon.
                </p>
            </div>

            <div className="space-y-3">
                <div className="flex items-center gap-3">
                    <button
                        type="button"
                        onClick={pickToml}
                        className="rounded-md bg-secondary px-4 py-2 text-sm font-medium text-secondary-foreground"
                    >
                        Select .toml
                    </button>

                    <span className="text-sm text-muted-foreground truncate">
                        {tomlPath ?? "No TOML file selected"}
                    </span>
                </div>

                <div className="flex items-center gap-3">
                    <button
                        type="button"
                        onClick={pickSo}
                        className="rounded-md bg-secondary px-4 py-2 text-sm font-medium text-secondary-foreground"
                    >
                        Select .so
                    </button>

                    <span className="text-sm text-muted-foreground truncate">
                        {soPath ?? "No SO file selected"}
                    </span>
                </div>
            </div>

            <button
                type="button"
                onClick={installPlugin}
                disabled={!tomlPath || !soPath || isInstalling}
                className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-50"
            >
                {isInstalling ? "Installing..." : "Install plugin"}
            </button>

            {status && (
                <p className="text-sm text-muted-foreground">
                    {status}
                </p>
            )}
        </div>
    );
}