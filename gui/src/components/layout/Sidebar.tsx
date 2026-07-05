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
import { useSidebar } from "@/providers/SidebarProvider.tsx";

export function Sidebar() {
  const { isCollapsed } = useSidebar();
  const { plugins, pluginStatus, refreshPlugins } = usePlugins();
  const location = useLocation();
  const [isRefreshing, setIsRefreshing] = useState(false);

  const handleRefresh = async () => {
    try {
      setIsRefreshing(true);

      await invoke("refresh_plugin");

      await new Promise((resolve) => setTimeout(resolve, 500));

      await refreshPlugins();
    } catch (error) {
      console.error("Failed to refresh Background Service:", error);
    } finally {
      setIsRefreshing(false);
    }
  };

  return (
    <aside
      className={`transition-all duration-200 ease-in-out flex flex-col gap-2 ${isCollapsed ? "w-min" : "w-48"
        } m-2 pl-2 pt-6 pb-2`}
    >
      <Link to="/dashboard">
        <SidebarButton
          icon={<LayoutDashboard />}
          label="Overview"
          isActive={location.pathname === "/dashboard" || location.pathname === "/"}
          isCollapsed={isCollapsed}
        />
      </Link>

      <Link to="/log">
        <SidebarButton
          icon={<Clock10 />}
          label="Activity Log"
          isActive={location.pathname === "/log"}
          isCollapsed={isCollapsed}
        />
      </Link>

      <SearchInput isCollapsed={isCollapsed} />

      <div>
        <Separator className="mt-2" />

        {!isCollapsed && (
          <span className="text-xs text-muted-foreground px-2 my-2 select-none">Extensions</span>
        )}
      </div>

      {plugins
        .filter((plugin) => pluginStatus[plugin.uuid] ?? true) // Only pass enabled plugins
        .map((plugin) => (
          <Link key={plugin.uuid} to={`/plugin/${plugin.file_name}`} className="block">
            <SidebarButton
              icon={<ToyBrick />}
              label={plugin.display_name}
              isActive={location.pathname === `/plugin/${plugin.file_name}`}
              isCollapsed={isCollapsed}
            />
          </Link>
        ))}

      <div className="flex-1" />

      <div
        className={
          isCollapsed ? "flex flex-col gap-2 justify-center" : "flex flex-row gap-2 justify-center"
        }
      >
        <Button
          title="Refresh Background Service"
          variant="outline"
          size="icon"
          className="cursor-pointer"
          disabled={isRefreshing}
          onClick={handleRefresh}
        >
          <RefreshCw className={isRefreshing ? "animate-spin" : ""} />
        </Button>

        <Link to="/settings">
          <Button variant="outline" size="icon" className="cursor-pointer" title="Settings">
            <Settings2 />
          </Button>
        </Link>

        <ModeToggle />
        <ContactButton />
      </div>
    </aside>
  );
}
