import { listen } from "@tauri-apps/api/event";
import { debugLog } from "./debugManager";

export type DaemonPlugin = {
    pid: number;
    status: boolean;
    plugin_uuid: number[];
    name: string;
    path: string;
    functions: string[];
};

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

type PluginsUpdatedEvent = {
    request_id: number;
    plugins: DaemonPlugin[];
};


const pending = new Map<number, PendingRequest>();

let initializationPromise: Promise<void> | null = null;

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
        debugLog("Rejecting request:", request_id, "with error:", value);
        entry.reject(value);
    } else {
        debugLog("Resolving request:", request_id, "with value:", value);
        entry.resolve(value);
    }
}

async function registerListeners(): Promise<void> {
    await Promise.all([
        listen<PluginCallEvent>("plugin-call-result", ({ payload }) => {
            resolvePendingRequest(
                payload.request_id,
                payload.ok ? payload.output : new Error(payload.output),
                !payload.ok
            );
        }),

        listen<PluginError>("plugin-error", ({ payload }) => {
            resolvePendingRequest(
                payload.request_id,
                new Error(payload.message),
                true
            );
        }),

        listen<PluginSwitchEvent>("plugin-switch-done", ({ payload }) => {
            resolvePendingRequest(payload.request_id, {
                enable: payload.enable,
            });
        }),

        listen<PluginsUpdatedEvent>("plugins-updated", ({ payload }) => {
            resolvePendingRequest(payload.request_id, payload.plugins);
        }),
    ]);
}

export async function initializeRequestManager() {
    if (initializationPromise) {
        return initializationPromise;
    }

    initializationPromise = registerListeners().catch((err) => {
        initializationPromise = null;
        throw err;
    });

    return initializationPromise;
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