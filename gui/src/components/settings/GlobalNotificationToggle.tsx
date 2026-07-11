import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Bell } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

type GlobalNotificationToggleProps = {
    onToggle?: () => void | Promise<void>;
};

export function GlobalNotificationToggle({ onToggle }: GlobalNotificationToggleProps) {
    const [globalNotifications, setGlobalNotifications] = useState<boolean>(true);
    const [isToggling, setIsToggling] = useState<boolean>(false);

    // Fetch initial status on mount
    useEffect(() => {
        invoke<boolean>("get_global_notification_status")
            .then(setGlobalNotifications)
            .catch(console.error);
    }, []);

    const handleToggle = async () => {
        try {
            setIsToggling(true);
            const newState = await invoke<boolean>("toggle_global_notifications");
            setGlobalNotifications(newState);

            if (onToggle) {
                await onToggle();
            }
        } catch (err) {
            console.error(err);
            alert("Failed to toggle global notifications.");
        } finally {
            setIsToggling(false);
        }
    };

    return (
        <div className="mt-6 mb-6 flex items-center justify-between rounded-md border-2 border-primary/20 bg-primary/5 p-4">
            <div className="flex items-center gap-2">

                {/* Your Tooltip Snippet */}
                <Tooltip>
                    <TooltipTrigger asChild>
                        <Button
                            className={`cursor-default hover:bg-transparent ${globalNotifications ? "text-primary" : "text-muted-foreground"}`}
                            variant={"ghost"}
                            size={"icon"}
                        >
                            <Bell />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                        <p>Notifications</p>
                    </TooltipContent>
                </Tooltip>

                <div>
                    <p className="text-base font-semibold text-foreground">Global Notifications</p>
                    <p className="mt-1 text-xs text-muted-foreground">
                        Master switch to enable or disable all notifications across Griffon.
                    </p>
                </div>
            </div>

            <Button
                variant={globalNotifications ? "secondary" : "default"}
                disabled={isToggling}
                onClick={handleToggle}
                className="cursor-pointer min-w-32"
            >
                {isToggling
                    ? "Updating..."
                    : globalNotifications
                        ? "Disable globally"
                        : "Enable globally"}
            </Button>
        </div>
    );
}