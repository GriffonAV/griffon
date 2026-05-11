import { listen } from "@tauri-apps/api/event";

type PendingRequest = {
    resolve: (value: any) => void;
    reject: (reason?: any) => void;
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

    await listen("plugin-call-result", (event) => {
        const { request_id, ok, output } = event.payload as any;

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

    await listen("plugin-switch-done", (event) => {
        const { request_id, enable } = event.payload as any;

        resolvePendingRequest(request_id, { enable });
    });
}

export function createPendingRequest<T = any>() {

    const requestId : number = generateRequestId();

    let timeout: ReturnType<typeof setTimeout>;

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

        }, 10000);
    });

    return {
        requestId,
        promise,
    };
}