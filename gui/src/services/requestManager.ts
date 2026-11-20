import { listen } from "@tauri-apps/api/event";
import { debugLog } from "./debugManager";

type PendingRequest = {
    resolve: (value: any) => void;
    reject: (reason?: any) => void;
};

type PluginCallEvent = {
    request_id: number;
    ok: boolean;
    output: string;
};

type PluginError = {
    request_id: number;
    message: string;
};

type PluginSwitchEvent = {
    request_id: number;
    enable: boolean;
};


const pending = new Map<number, PendingRequest>();

let initialized = false;

function generateRequestId(): number {
    return window.crypto.getRandomValues(new Uint32Array(1))[0];

    // return crypto.randomUUID(); --- To implement ---
}

function resolvePendingRequest(
    request_id: number,
    value: any,
    isError = false
) {
    const entry = pending.get(request_id);

    if (!entry) {
        console.warn("Unknown request:", request_id);
        return;
    }

    pending.delete(request_id);

    if (isError) {
        entry.reject(value);
    } else {
        entry.resolve(value);
    }
}

export async function initializeRequestManager() {

    if (initialized) {
        return;
    }

    initialized = true;

    await listen<PluginCallEvent>("plugin-call-result", (event) => {
        const { request_id, ok, output } = event.payload;

        if (ok) {
            resolvePendingRequest(request_id, output);
        } else {
            resolvePendingRequest(
                request_id,
                new Error(output),
                true
            );
        }
    });

    await listen<PluginError>("plugin-error", (event) => {
        const { request_id, message } = event.payload;

        resolvePendingRequest(
            request_id,
            new Error(message),
            true
        );
    });

    await listen<PluginSwitchEvent>("plugin-switch-done", (event) => {
        const { request_id, enable } = event.payload;

        resolvePendingRequest(request_id, { enable });
    });
}

export function createPendingRequest<T = any>() {

    const requestId: number = generateRequestId();

    let timeout: ReturnType<typeof setTimeout>;

    debugLog("Creating pending request:", requestId, "Pending requests count:", pending.size);

    const promise = new Promise<T>((resolve, reject) => {

        pending.set(requestId, {
            resolve: (value) => {
                clearTimeout(timeout);
                resolve(value);
            },

            reject: (reason) => {
                clearTimeout(timeout);
                reject(reason);
            },
        });

        timeout = setTimeout(() => {

            if (!pending.has(requestId)) {
                return;
            }

            pending.delete(requestId);

            reject(
                new Error(
                    `Request timed out (${requestId})`
                )
            );

        }, 6000000);
    });

    return {
        requestId,
        promise,
    };
}