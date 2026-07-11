import { useState } from "react";
import { Link } from "react-router-dom";
import { ToyBrick } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

import { usePlugins } from "@/bindings/PluginContext.tsx";
import { Button } from "../ui/button";
import { createPendingRequest } from "@/services/requestManager";

type PluginSwitchDonePayload = {
    request_id: string;
    enable: boolean;
};

async function switchPluginStatus(
    pluginUuid: string
): Promise<PluginSwitchDonePayload> {
    const { requestId, promise } = createPendingRequest();

    await invoke("switch_status_plugin", {
        pluginUuid,
        requestId,
    });

    return promise;
}

export function PluginToggleSettings() {
    const { plugins, pluginStatus, setPluginStatus } = usePlugins();
    // const [pluginStatus, setPluginStatus] = useState<Record<string, boolean>>({});
    const [switchingPlugins, setSwitchingPlugins] = useState<Record<string, boolean>>({});

    async function handleSwitchPlugin(pluginUuid: string) {
        try {
            setSwitchingPlugins((prev) => ({
                ...prev,
                [pluginUuid]: true,
            }));

            const result = await switchPluginStatus(pluginUuid);

            setPluginStatus((prev) => ({
                ...prev,
                [pluginUuid]: result.enable,
            }));
        } catch (error) {
            console.error("Failed to switch plugin status:", error);
        } finally {
            setSwitchingPlugins((prev) => ({
                ...prev,
                [pluginUuid]: false,
            }));
        }
    }

    if (plugins.length === 0) {
        return (
            <p className="m-5 text-sm text-muted-foreground">
                No extensions available.
            </p>
        );
    }

    return (
        <div className="flex flex-col gap-3 w-full">
            {plugins.map((plugin) => {
                const isEnabled = pluginStatus[plugin.uuid] ?? true;
                const isSwitching = switchingPlugins[plugin.uuid] ?? false;

                return (
                    <div
                        key={plugin.uuid}
                        className="flex items-center justify-between gap-4 rounded-md border p-4"
                    >
                        <div className="flex items-center gap-3 min-w-0">
                            <ToyBrick className="shrink-0" />

                            <div>
                                <Link
                                    to={`/plugin/${plugin.file_name}`}
                                    className="text-sm text-muted-foreground hover:underline"
                                >
                                    <p className="text-base font-semibold">{plugin.display_name}</p>
                                </Link>
                                <p className="mt-1 text-xs text-muted-foreground">
                                    {plugin.description || "No description available."}
                                </p>
                                <p className="mt-1 text-xs text-muted-foreground">
                                    {plugin.file_name} • Version {plugin.version} • {plugin.author}
                                </p>
                            </div>
                        </div>

                        <Button
                            variant="outline"
                            disabled={isSwitching}
                            className="min-w-28 cursor-pointer gap-2"
                            title={isEnabled ? "Disable plugin" : "Enable plugin"}
                            onClick={() => handleSwitchPlugin(plugin.uuid)}
                        >
                            <span
                                className={
                                    isEnabled ? "text-green-500" : "text-red-500"
                                }
                            >
                                ●
                            </span>

                            {isSwitching
                                ? "Switching..."
                                : isEnabled
                                    ? "Enabled"
                                    : "Disabled"}
                        </Button>
                    </div>
                );
            })}
        </div >
    );
}