import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { MonitorUp } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { invoke } from '@tauri-apps/api/core'; // or '@tauri-apps/api/tauri' depending on version

function DaemonConnection() {
    const [isConnected, setIsConnected] = useState<boolean>(false);

    useEffect(() => {
        let unlistenStatus: any = null;

        const init = async () => {
            // 1. Check current status immediately
            const initialStatus = await invoke<boolean>('get_daemon_status');
            setIsConnected(initialStatus);

            // 2. Listen for future changes
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
        <Button
            className={`cursor-pointer ${isConnected ? 'text-green-500' : 'text-muted-foreground opacity-50'}`}
            variant="ghost"
        >
            <MonitorUp />
        </Button>
    );
}

export default DaemonConnection;