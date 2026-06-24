import { Button } from "@/components/ui/button";
import clsx from "clsx";
import type { ReactNode } from "react";

interface SidebarButtonProps {
  icon: ReactNode;
  label: string;
  isActive: boolean;
  isCollapsed?: boolean;
}

export function SidebarButton({ icon, label, isActive, isCollapsed }: SidebarButtonProps) {
  return (
    <Button
      title={label}
      variant="ghost"
      className={clsx(
        "w-full cursor-pointer font-bold overflow-hidden",
        isCollapsed ? "justify-center px-0" : "justify-start",
        isActive && "bg-sidebar-primary text-sidebar-primary-foreground"
      )}
    >


      <span className={clsx(isCollapsed ? "mr-0" : "mr-2")}>
        {icon}
      </span>

      {!isCollapsed && (
        <span className={"transition-transform duration-200"}>
          {label}
        </span>
      )}
    </Button>
  );
}