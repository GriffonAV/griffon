import { Link, useLocation } from "react-router-dom";
import { usePlugins } from "@/hooks/usePlugins";
import clsx from "clsx";
import { ModeToggle } from "./mode-toggle";
import { Settings2, LayoutDashboard, Logs } from "lucide-react";
import { SearchInput } from "./search";
import { Info } from "./info";
import { SidebarButton } from "./sidebar-button";
import SidebarNotifications from "./sidebar-notifications.tsx";
import { Button } from "./ui/button.tsx";

export function Sidebar() {
  const { plugins } = usePlugins();
  const location = useLocation();

  return (
    <aside className="flex flex-col w-48 m-2">
      <Link to="/dashboard">
        <SidebarButton
          to="/dashboard"
          icon={<LayoutDashboard />}
          label="Dashboard"
          isActive={location.pathname === "/dashboard" || location.pathname === "/"}
        />
      </Link>
      <SidebarNotifications />
      <Link to="/log">
        <SidebarButton
          to="/log"
          icon={<Logs />}
          label="Logs"
          isActive={location.pathname === "/log"}
        />
      </Link>
      <SearchInput />

      <span className="text-xs text-muted-foreground px-2 my-2 select-none">
        Plugins
      </span>
      {plugins.map((plugin) => (
        <Link key={plugin.pid} to={`/plugin/${plugin.pid}`}>
          <SidebarButton
            to={`/plugin/${plugin.pid}`}
            icon={null}
            label={plugin.name}
            isActive={location.pathname === `/plugin/${plugin.pid}`}
          />
        </Link>
      ))}

      {/* <Link to="/settings">
        <SidebarButton
          to="/settings"
          icon={<Settings2 />}
          label="Settings"
          isActive={location.pathname === "/settings"}
        />
      </Link> */}

      <div className="flex-1" />

      <div className="flex flex-row gap-2 justify-end">
        <Link to="/settings">
          <Button variant="outline" size="icon" className="cursor-pointer">
            <Settings2 ></Settings2>
            <span className="sr-only">Settings</span>
          </Button>
        </Link>
        <ModeToggle />
        <Info />

      </div>
    </aside>
  );
}