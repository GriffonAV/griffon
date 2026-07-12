import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { MonitorUp } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { invoke } from '@tauri-apps/api/core';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';

type DaemonStatusEvent = {
    status: string;
    error?: string;
};

function DaemonConnection() {
    const [isConnected, setIsConnected] = useState<boolean>(false);
    const [errorMsg, setErrorMsg] = useState<string>('');

    useEffect(() => {
        let unlistenStatus: any = null;

        const init = async () => {
            // Check initial status
            const initialStatus = await invoke<boolean>('get_daemon_status');
            setIsConnected(initialStatus);

            // Listen for status changes
            unlistenStatus = await listen<DaemonStatusEvent | string>('daemon-status', (event) => {
                console.log("Daemon status event received:", event.payload); // Helps with debugging

                if (typeof event.payload === 'object' && event.payload !== null) {
                    setIsConnected(event.payload.status === "Connected");
                    setErrorMsg(event.payload.error || "");
                } else {
                    // Fallback just in case a plain string is received
                    setIsConnected(event.payload === "Connected");
                    setErrorMsg("");
                }
            });
        };

        init();

        return () => {
            if (unlistenStatus) unlistenStatus();
        };
    }, []);

    // 1. Add this function to call the backend
    const handleForceReconnect = async () => {
        try {
            await invoke('force_reconnect');
        } catch (error) {
            console.error("Failed to force reconnect:", error);
        }
    };

    return (
        <Tooltip>
            <TooltipTrigger asChild>
                {/* 2. Bind the function to the onClick event */}
                <Button
                    onClick={handleForceReconnect}
                    className={`cursor-pointer ${isConnected ? 'text-green-500' : 'text-muted-foreground opacity-50'}`}
                    variant="ghost"
                >
                    <MonitorUp />
                </Button>
            </TooltipTrigger>
            <TooltipContent>
                {isConnected ? (
                    <p>Background Service is connected. Click to restart connection.</p>
                ) : (
                    <div className="flex flex-col gap-1 max-w-xs">
                        <p className="font-semibold text-red-500">
                            Background Service disconnected. Click to reconnect.
                        </p>
                        {errorMsg && (
                            <p className="text-xs text-muted-foreground break-words">{errorMsg}</p>
                        )}
                    </div>
                )}
            </TooltipContent>
        </Tooltip>
    );
}

export default DaemonConnection;