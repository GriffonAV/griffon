import { useEffect, useMemo, useState } from "react";
import type { PluginManifest, InteractionStep } from "@/bindings/PluginContext";

export type GriffonStore = Record<string, any>;

function resolveFromPath(
  path: string | null | undefined,
  context: { store: GriffonStore; event?: any }
): any {
  // defensive valitdation for potential use as standalone function
  if (typeof path !== "string") return undefined;

  const trimmed = path.trim();
  if (!trimmed) {
    console.warn("Empty path provided for resolveFromPath");
    return undefined;
  }

  const parts = trimmed.split(".");
  let current: any = context;
  console.debug(`Resolving path "${path}" with context:`, context);
  console.debug(`current:`, current);

  for (const part of parts) {
    current = current?.[part];
    if (current === undefined) {
        console.warn(`Path "${path}" is invalid at segment "${part}"`);
        return undefined;
    }
  }

  return current;
}

function executeStep(
  draft: GriffonStore,
  step: InteractionStep,
  event?: any
): GriffonStore {
  const next = { ...draft };

  switch (step.type) {
    case "set": {
      if (!step.key)
        return next;
      
      const hasFrom = typeof step.from === "string" && step.from.trim().length > 0;
      const value = hasFrom
          ? resolveFromPath(step.from, { store: next, event })
          : step.value;

      next[step.key] = value;
      return next;
    }

    case "increment": {
      if (!step.key)
        return next;

      const current = Number(next[step.key] ?? 0);
      const amount = Number(step.amount ?? 1);

      next[step.key] = current + amount;
      return next;
    }

    case "decrement": {
      if (!step.key)
        return next;

      const current = Number(next[step.key] ?? 0);
      const amount = Number(step.amount ?? 1);

      next[step.key] = current - amount;
      return next;
    }

    case "toggle": {
      if (!step.key)
        return next;

      next[step.key] = !Boolean(next[step.key]);
      return next;
    }

    default: {
      console.warn(`Unknown step type: ${(step as any).type}`);
      return next;
    }
  }
}

export function useGriffonStore(manifest: PluginManifest | null) {
  const initialStore = useMemo(() => manifest?.store ?? {}, [manifest]);
  const [store, setStore] = useState<GriffonStore>(initialStore);

  useEffect(() => {
    setStore(manifest?.store ?? {});
  }, [manifest]);

  function handleAction(action: string, event?: any) {
    if (!manifest?.interactions?.length) {
        console.warn("No interactions defined in manifest");
        return;
    }

    const matching = manifest.interactions.filter((interaction) => interaction.on === action);
    if (!matching.length) {
        console.warn(`No interactions found for action: ${action}`);
        return;
    }

    setStore((prev) => {
      let next = { ...prev };

      for (const interaction of matching) {
        for (const step of interaction.steps ?? []) {
          next = executeStep(next, step, event);
        }
      }

      return next;
    });
  }

  function setValue(key: string, value: any) {
    setStore((prev) => ({
      ...prev,
      [key]: value,
    }));
  }

  return {
    store,
    handleAction,
    setValue,
  };
}