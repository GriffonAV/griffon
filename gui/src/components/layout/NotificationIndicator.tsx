import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Bell, BellOff } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

export function NotificationIndicator() {
    const [notificationsEnabled, setNotificationsEnabled] = useState(true);

    useEffect(() => {
        const fetchStatus = async () => {
            try {
                const status = await invoke<boolean>("get_global_notification_status");
                setNotificationsEnabled(status);
            } catch (error) {
                console.error("Failed to fetch notification status:", error);
            }
        };

        fetchStatus();
    }, []);

    return (
        <Tooltip>
            <TooltipTrigger asChild>
                {/* Later, you can wrap this Button in a DropdownMenuTrigger or PopoverTrigger for the notification list */}
                <Button className="cursor-pointer text-muted-foreground" variant="ghost">
                    {notificationsEnabled ? <Bell /> : <BellOff />}
                </Button>
            </TooltipTrigger>
            <TooltipContent>
                <p>{notificationsEnabled ? "Notifications enabled" : "Notifications disabled"}</p>
            </TooltipContent>
        </Tooltip>
    );
}