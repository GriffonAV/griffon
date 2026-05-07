import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

// USE TO HANDLE IN THE FRONT THE ANSWER FROM THE DAEMON
export function useDaemonNotifications() {
    useEffect(() => {
        let unlisten: (() => void) | null = null;

        const setup = async () => {
            unlisten = await listen("daemon-ok", (event) => {
                console.log("daemon-ok reçu :", event.payload);
                alert(`daemon-ok reçu: ${JSON.stringify(event.payload)}`);
            });
        };

        setup();

        return () => {
            if (unlisten) {
                unlisten();
            }
        };
    }, []);
}