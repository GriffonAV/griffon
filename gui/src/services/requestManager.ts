import { listen } from "@tauri-apps/api/event";
import {
    createPendingRequest,
    rejectPendingRequest,
    resolvePendingRequest,
} from "./requestManagerCore";

let initialized = false;

export { createPendingRequest };

export async function initializeRequestManager() {

    if (initialized) return;

    initialized = true;

    await listen("plugin-call-result", (event) => {
        const { request_id, ok, output } = event.payload as {
            request_id: number;
            ok: boolean;
            output: string;
        };

        if (ok) {
            resolvePendingRequest(request_id, output);
        } else {
            rejectPendingRequest(request_id, new Error(output));
        }
    });

    await listen("plugin-switch-done", (event) => {
        const { request_id, enable } = event.payload as {
            request_id: number;
            enable: boolean;
        };

        resolvePendingRequest(request_id, { enable });
    });
}
