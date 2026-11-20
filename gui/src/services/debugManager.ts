const debugEnabled =
  import.meta.env.DEV &&
  import.meta.env.VITE_NETWORK_DEBUG === "true";

export function debugLog(...values: unknown[]): void {
  if (debugEnabled) {
    console.debug("[GUI-NETWORK]", ...values);
  }
}