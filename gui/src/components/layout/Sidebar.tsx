import { useState } from "react";
import { Link, useLocation } from "react-router-dom";
import { usePlugins } from "@/bindings/PluginContext.tsx";
import { ModeToggle } from "./ModeToggle.tsx";
import { Settings2, LayoutDashboard, Clock10, RefreshCw, ToyBrick } from "lucide-react";
import { SearchInput } from "./SearchInput.tsx";
import { ContactButton } from "./ContactButton.tsx";
import { SidebarButton } from "./SidebarButton.tsx";
import { Button } from "../ui/button.tsx";
import { Separator } from "@/components/ui/separator.tsx";
import { invoke } from "@tauri-apps/api/core";
import { createPendingRequest } from "@/services/requestManager";
import { useSidebar } from "@/providers/SidebarProvider.tsx";

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

export function Sidebar() {
    const { isCollapsed } = useSidebar();
    const { plugins } = usePlugins();
    const location = useLocation();
    const [isRefreshing, setIsRefreshing] = useState(false);
    const [pluginStatus, setPluginStatus] = useState<Record<string, boolean>>({});
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

    return (
        // <aside className="transition-all duration-200 ease-in-out flex flex-col w-48 m-2">
        <aside className={`transition-all duration-200 ease-in-out flex flex-col gap-2 ${isCollapsed ? "w-min" : "w-48"} m-2 pl-2 pt-6 pb-2`}>

            <Link to="/dashboard">
                <SidebarButton
                    icon={<LayoutDashboard />}
                    label="Dashboard"
                    isActive={location.pathname === "/dashboard" || location.pathname === "/"}
                    isCollapsed={isCollapsed}
                />
            </Link>

            <Link to="/log">
                <SidebarButton
                    icon={<Clock10 />}
                    label="Logs"
                    isActive={location.pathname === "/log"}
                    isCollapsed={isCollapsed}
                />
            </Link>

            <SearchInput isCollapsed={isCollapsed} />

            <div >

                <Separator className="mt-2" />

                {!isCollapsed &&

                    <span className="text-xs text-muted-foreground px-2 my-2 select-none">
                        Plugins
                    </span>
                }
            </div>

            {plugins.map((plugin) => {
                const isEnabled = pluginStatus[plugin.uuid] ?? true;
                const isSwitching = switchingPlugins[plugin.uuid] ?? false;

                return (
                    <div key={plugin.uuid} className="relative">
                        <Link to={`/plugin/${plugin.file_name}`} className="block">
                            <SidebarButton
                                icon={<ToyBrick />}
                                label={plugin.display_name}
                                isActive={location.pathname === `/plugin/${plugin.file_name}`}
                                isCollapsed={isCollapsed}
                            />
                        </Link>


                        {!isCollapsed && (
                            <Button
                                variant="ghost"
                                size="icon"
                                disabled={isSwitching}
                                className={`absolute top-1 right-1 z-10 h-6 w-6 rounded-full bg-transparent p-0 shadow-none hover:bg-transparent ${isEnabled ? "text-green-500 hover:text-green-600" : "text-red-500 hover:text-red-600"
                                    }`}
                                title={isEnabled ? "Disable plugin" : "Enable plugin"}
                                onClick={() => handleSwitchPlugin(plugin.uuid)}
                            >
                                <span className="text-base leading-none">●</span>
                            </Button>
                        )}
                    </div>
                );
            })}

            <div className="flex-1" />

            <div className={isCollapsed ? "flex flex-col gap-2 justify-center" : "flex flex-row gap-2 justify-center"}>
                <Button
                    title="Refresh daemon"
                    variant="outline"
                    size="icon"
                    className="cursor-pointer"
                    disabled={isRefreshing}
                    onClick={async () => {
                        try {
                            setIsRefreshing(true);

                            await Promise.all([
                                invoke("refresh_plugin"),
                                new Promise((resolve) => setTimeout(resolve, 500)),
                            ]);
                        } catch (error) {
                            console.error("Failed to refresh plugins:", error);
                        } finally {
                            setIsRefreshing(false);
                        }
                    }}
                >
                    <RefreshCw className={isRefreshing ? "animate-spin" : ""} />
                    <span className="sr-only">Refresh plugins</span>
                </Button>

                <Link to="/settings">
                    <Button variant="outline" size="icon" className="cursor-pointer" title="Settings">
                        <Settings2 />
                        {/* <span className="sr-only">Settings</span> */}
                    </Button>
                </Link>

                <ModeToggle />
                <ContactButton />
            </div>
        </aside>
    );
}