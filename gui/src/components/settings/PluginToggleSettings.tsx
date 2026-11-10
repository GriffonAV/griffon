import { Link } from "react-router-dom";
import { ToyBrick } from "lucide-react";

import { usePlugins } from "@/bindings/PluginContext.tsx";
import { Button } from "../ui/button";


export function PluginToggleSettings() {
    const { plugins, switchPluginStatus } = usePlugins();

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
                const isEnabled = plugin.status === "healthy";

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
                            disabled={plugin.status === "busy"}
                            className="min-w-28 cursor-pointer gap-2"
                            title={isEnabled ? "Disable plugin" : "Enable plugin"}
                            onClick={() => switchPluginStatus(plugin.uuid)}
                        >
                            <span
                                className={
                                    isEnabled ? "text-green-500" : "text-red-500"
                                }
                            >
                                ●
                            </span>

                            {plugin.status === "busy"
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