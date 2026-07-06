import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

type InstalledPlugin = {
    toml_path: string;
    so_path: string;
    plugin_dir: string;
};

type PluginInstallerProps = {
    onInstalled?: () => void | Promise<void>;
};

export function PluginInstaller({ onInstalled }: PluginInstallerProps) {
    const [zipPath, setZipPath] = useState<string | null>(null);
    const [status, setStatus] = useState<string | null>(null);
    const [isInstalling, setIsInstalling] = useState(false);

    const pickZip = async () => {
        const selected = await open({
            multiple: false,
            filters: [
                {
                    name: "Griffon Plugin Archive",
                    extensions: ["zip"],
                },
            ],
        });

        if (typeof selected === "string") {
            setZipPath(selected);
            setStatus(null);
        }
    };

    const installPlugin = async () => {
        if (!zipPath) {
            setStatus("Please select a plugin .zip file.");
            return;
        }

        try {
            setIsInstalling(true);
            setStatus("Installing plugin...");

            // Make sure to match this new command name in your Rust backend
            const result = await invoke<InstalledPlugin>("install_plugin_zip", {
                zipPath,
            });

            setStatus(`Plugin installed successfully in ${result.plugin_dir}. Refreshing plugin list...`);

            try {
                await onInstalled?.();
                setZipPath(null);
                setStatus(`Plugin installed successfully in ${result.plugin_dir}.`);
            } catch (refreshError) {
                console.error(refreshError);
                setStatus(
                    `Plugin installed successfully in ${result.plugin_dir}, but the plugin list could not be refreshed.`
                );
            }
        } catch (error) {
            setStatus(`Installation failed: ${String(error)}`);
        } finally {
            setIsInstalling(false);
        }
    };

    return (
        <div>
            <div className="space-y-3 mb-3">
                <div className="flex items-center gap-3">
                    <button
                        type="button"
                        onClick={pickZip}
                        className="rounded-md bg-secondary px-4 py-2 text-sm font-medium text-secondary-foreground"
                    >
                        Select .zip
                    </button>

                    <span className="text-sm text-muted-foreground truncate">
                        {zipPath ?? "No ZIP file selected"}
                    </span>
                </div>
            </div>

            <button
                type="button"
                onClick={installPlugin}
                disabled={!zipPath || isInstalling}
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