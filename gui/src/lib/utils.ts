import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function resolveTemplate(value: string, context: { store: any; event?: any }) {
  return value.replace(/\{\{(.*?)\}\}/g, (_, rawKey) => {
    const path = String(rawKey).trim();
    const resolved = resolveFromPath(path, context);
    return resolved !== undefined && resolved !== null ? String(resolved) : "";
  });
}

export function resolveFromPath(
  path: string | null | undefined,
  context: { store: Record<string, any>; event?: any }
): any {
  if (typeof path !== "string") return undefined;

  let trimmed = path.trim();
  if (!trimmed) return undefined;

  let current: any;

  if (trimmed.startsWith("store.")) {
    current = context.store;
    trimmed = trimmed.slice("store.".length);
  } else if (trimmed.startsWith("event.")) {
    current = context.event;
    trimmed = trimmed.slice("event.".length);
  } else {
    current = context.store;
  }

  const parts = trimmed.split(".");

  for (const part of parts) {
    if (!part) continue;
    current = current?.[part];
    if (current === undefined) return undefined;
  }

  return current;
}