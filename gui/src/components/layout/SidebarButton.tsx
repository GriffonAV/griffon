import { Button } from "@/components/ui/button";
import clsx from "clsx";
import type { ReactNode } from "react";

interface SidebarButtonProps {
  icon: ReactNode;
  label: string;
  isActive: boolean;
}

export function SidebarButton({ icon, label, isActive }: SidebarButtonProps) {
  return (
    <Button
      title={label}
      variant="ghost"
      className={clsx(
        "w-full justify-start cursor-pointer mb-2 font-bold overflow-hidden",
        isActive && "bg-sidebar-primary text-sidebar-primary-foreground"
      )}
    >
      <span className="mr-2">{icon}</span>
      {label}
    </Button>
  );
}
