import { useEffect, useRef, useState } from "react";
import { Link, useLocation } from "react-router-dom";
import { usePlugins } from "@/bindings/PluginContext.tsx";
import { ModeToggle } from "./ModeToggle.tsx";
import { Settings2, LayoutDashboard, Clock10 } from "lucide-react";
import { SearchInput } from "./SearchInput.tsx";
import { ContactButton } from "./ContactButton.tsx";
import { SidebarButton } from "./SidebarButton.tsx";
import { Button } from "../ui/button.tsx";
import { Separator } from "@/components/ui/separator.tsx";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type PluginSwitchDonePayload = {
    request_id: number;
    enable: boolean;
};

export function Sidebar() {
    const { plugins } = usePlugins();
    const location = useLocation();

    const [pluginStatus, setPluginStatus] = useState<Record<string, boolean>>({});
    const pendingSwitchRequests = useRef<Record<number, string>>({});

    useEffect(() => {
        const unlistenPromise = listen<PluginSwitchDonePayload>(
            "plugin-switch-done",
            (event) => {
                const { request_id, enable } = event.payload;
                const pluginUuid = pendingSwitchRequests.current[request_id];

                if (!pluginUuid) {
                    console.warn("Unknown switch request:", request_id);
                    return;
                }

                setPluginStatus((previous) => ({
                    ...previous,
                    [pluginUuid]: enable,
                }));

                delete pendingSwitchRequests.current[request_id];
            }
        );

        return () => {
            unlistenPromise.then((unlisten) => unlisten());
        };
    }, []);

    return (
        <aside className="flex flex-col w-48 m-2">
            <Link to="/dashboard">
                <SidebarButton
                    icon={<LayoutDashboard />}
                    label="Dashboard"
                    isActive={location.pathname === "/dashboard" || location.pathname === "/"}
                />
            </Link>

            <Link to="/log">
                <SidebarButton
                    icon={<Clock10 />}
                    label="Logs"
                    isActive={location.pathname === "/log"}
                />
            </Link>

            <SearchInput />
            <Separator />

            <span className="text-xs text-muted-foreground px-2 my-2 select-none">
        Plugins
      </span>

            {plugins.map((plugin) => {
                const isEnabled = pluginStatus[plugin.uuid] ?? true;

                return (
                    <div key={plugin.uuid} className="flex items-center gap-1">
                        <Link to={`/plugin/${plugin.file_name}`} className="flex-1 min-w-0">
                            <SidebarButton
                                icon={null}
                                label={plugin.display_name}
                                isActive={location.pathname === `/plugin/${plugin.file_name}`}
                            />
                        </Link>

                        <Button
                            variant="ghost"
                            size="icon"
                            className={`h-8 w-8 shrink-0 cursor-pointer ${
                                isEnabled
                                    ? "text-green-500 hover:text-green-600"
                                    : "text-red-500 hover:text-red-600"
                            }`}
                            title={isEnabled ? "Disable plugin" : "Enable plugin"}
                            onClick={async () => {
                                try {
                                    const requestId = await invoke<number>("switch_status_plugin", {
                                        pluginUuid: plugin.uuid,
                                    });

                                    pendingSwitchRequests.current[requestId] = plugin.uuid;
                                } catch (error) {
                                    console.error("Failed to switch plugin status:", error);
                                }
                            }}
                        >
                            ●
                        </Button>
                    </div>
                );
            })}

            <div className="flex-1" />

            <div className="flex flex-row gap-2 justify-end">
                <Button
                    className="cursor-pointer"
                    onClick={async () => {
                        try {
                            await invoke("refresh_plugin");
                        } catch (error) {
                            console.error("Failed to refresh plugins:", error);
                        }
                    }}
                >
                    R
                </Button>

                <Link to="/settings">
                    <Button variant="outline" size="icon" className="cursor-pointer">
                        <Settings2 />
                        <span className="sr-only">Settings</span>
                    </Button>
                </Link>

                <ModeToggle />
                <ContactButton />
            </div>
        </aside>
    );
}