import { Button } from "@/components/ui/button";
import clsx from "clsx";
import { ReactNode } from "react";

interface SidebarButtonProps {
    to: string;
    icon: ReactNode;
    label: string;
    isActive: boolean;
}

export function SidebarButton({ to, icon, label, isActive }: SidebarButtonProps) {
    return (
        <a href={to}>
            <Button
                variant="ghost"
                className={clsx(
                    "w-full justify-start cursor-pointer mb-2",
                    isActive && "bg-sidebar-primary text-sidebar-primary-foreground",
                )}
            >
                <span className="mr-2">{icon}</span>
                {label}
            </Button>
        </a>
    );
}