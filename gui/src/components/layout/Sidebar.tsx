import { Link, useLocation } from "react-router-dom";
import { usePlugins } from "@/bindings/PluginContext.tsx";
import { ModeToggle } from "./ModeToggle.tsx";
import { Settings2, LayoutDashboard, Clock10 } from "lucide-react";
import { SearchInput } from "./SearchInput.tsx";
import { ContactButton } from "./ContactButton.tsx";
import { SidebarButton } from "./SidebarButton.tsx";
import { Button } from "../ui/button.tsx";
import { Separator } from "@/components/ui/separator.tsx";
import {invoke} from "@tauri-apps/api/core";

export function Sidebar() {
  const { plugins } = usePlugins();
  const location = useLocation();

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
      <span className="text-xs text-muted-foreground px-2 my-2 select-none">Plugins</span>
      {plugins.map((plugin) => (
        <Link key={plugin.name} to={`/plugin/${plugin.name}`}>
          <SidebarButton
            icon={null}
            label={plugin.name}
            isActive={location.pathname === `/plugin/${plugin.name}`}
          />
        </Link>
      ))}

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
              <Settings2></Settings2>
              <span className="sr-only">Settings</span>
          </Button>
        </Link>
          <ModeToggle />
          <ContactButton />
      </div>
    </aside>
  );
}
