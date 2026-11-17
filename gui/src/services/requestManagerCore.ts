export const DEFAULT_REQUEST_TIMEOUT_MS = 6_000_000;

type PendingEntry = {
  resolve: (value: unknown) => void;
  reject: (reason?: unknown) => void;
  timeout: ReturnType<typeof setTimeout>;
};

const pending = new Map<number, PendingEntry>();

function generateRequestId(): number {
  const cryptoApi = globalThis.crypto;
  if (cryptoApi) return cryptoApi.getRandomValues(new Uint32Array(1))[0];
  return Math.floor(Math.random() * 2 ** 32);
}

export function resolvePendingRequest(requestId: number, value: unknown): boolean {
  const entry = pending.get(requestId);
  if (!entry) return false;
  pending.delete(requestId);
  clearTimeout(entry.timeout);
  entry.resolve(value);
  return true;
}

export function rejectPendingRequest(requestId: number, reason: unknown): boolean {
  const entry = pending.get(requestId);
  if (!entry) return false;
  pending.delete(requestId);
  clearTimeout(entry.timeout);
  entry.reject(reason);
  return true;
}

export function createPendingRequest<T = unknown>(
  timeoutMs = DEFAULT_REQUEST_TIMEOUT_MS,
) {
  const requestId = generateRequestId();
  let timeout: ReturnType<typeof setTimeout>;
  const promise = new Promise<T>((resolve, reject) => {
    timeout = setTimeout(() => {
      rejectPendingRequest(
        requestId,
        new Error(`Request timed out (${requestId})`),
      );
    }, timeoutMs);
    pending.set(requestId, {
      resolve: resolve as (value: unknown) => void,
      reject,
      timeout,
    });
  });

  return {
    requestId,
    promise,
    cancel: () =>
      rejectPendingRequest(requestId, new Error("Request cancelled")),
  };
}
