import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { MonitorUp } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { invoke } from '@tauri-apps/api/core';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';

function DaemonConnection() {
    const [isConnected, setIsConnected] = useState<boolean>(false);

    useEffect(() => {
        let unlistenStatus: any = null;

        const init = async () => {
            const initialStatus = await invoke<boolean>('get_daemon_status');
            setIsConnected(initialStatus);

            unlistenStatus = await listen<string>('daemon-status', (event) => {
                setIsConnected(event.payload === "Connected");
            });
        };

        init();

        return () => {
            if (unlistenStatus) unlistenStatus();
        };
    }, []);

    return (
        <Tooltip>
            <TooltipTrigger asChild>
                <Button
                    className={`cursor-pointer ${isConnected ? 'text-green-500' : 'text-muted-foreground opacity-50'}`}
                    variant="ghost"
                >
                    <MonitorUp />
                </Button>
            </TooltipTrigger>
            <TooltipContent>
                <p>{isConnected ? 'Daemon is connected' : 'Daemon is not connected'}</p>
            </TooltipContent>
        </Tooltip>

    );
}

export default DaemonConnection;