import { useEffect, useMemo, useState } from "react";
import type { PluginManifest, InteractionStep } from "@/bindings/PluginContext";
import { usePlugins } from "@/bindings/PluginContext";
import { resolveFromPath } from "@/lib/utils";

export type GriffonStore = Record<string, any>;


function setByPath(
  obj: Record<string, any>,
  path: string | null | undefined,
  value: any
) {
  if (typeof path !== "string") return false;

  let trimmed = path.trim();
  if (!trimmed) return false;

  if (trimmed.startsWith("event.")) {
    console.warn(`setByPath: cannot write to event ("${path}")`);
    return false;
  }

  if (trimmed.startsWith("store.")) {
    trimmed = trimmed.slice("store.".length);
  }

  const parts = trimmed.split(".").filter(Boolean);
  if (!parts.length) return false;

  let current: any = obj;

  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i];

    if (
      current[part] === undefined ||
      typeof current[part] !== "object" ||
      current[part] === null
    ) {
      console.warn(`setByPath: path does not exist ("${path}")`);
      return false;
    }

    current = current[part];
  }

  const lastKey = parts[parts.length - 1];

  if (!(lastKey in current)) {
    console.warn(`setByPath: key does not exist ("${path}")`);
    return false;
  }

  current[lastKey] = value;
  return true;
}


async function CallPlgFnStep(
    step: InteractionStep,
    next: GriffonStore,
    callPluginFunction: (fnName: string, args: string[]) => Promise<any>
): Promise<GriffonStore> {

    if (step.fn === undefined) {
        console.warn("fn param is required for execute_function step");
        return next;
    }
    if (typeof step.fn !== "string") {
        console.warn("fn param must be a string");
        return next;
    }

    try {
        const returnValue = await callPluginFunction(step.fn, step.args ?? []);
        let result : string = returnValue;

        if (step.returnType === "json") {
            try {
                result = JSON.parse(returnValue);
            } catch {
                result = returnValue;
            }
        }

        if (step.key) {
          setByPath(next, step.key, result);
        }

        return next;
    } catch (err) {
        console.error("Error executing plugin function:", err);
        return next;
    }
}



async function executeStep(
  draft: GriffonStore,
  step: InteractionStep,
  callPluginFunction: (fnName: string, args: string[]) => Promise<any>,
  event?: any,
): Promise<GriffonStore> {
  const next = { ...draft };

  switch (step.type) {
    case "set": {
      if (!step.key) return next;

      const hasFrom = typeof step.from === "string" && step.from.trim().length > 0;

      const value = hasFrom
        ? resolveFromPath(step.from, { store: next, event })
        : step.value;

      setByPath(next, step.key, value);
      return next;
    }

    case "increment": {
      if (!step.key) return next;

      const current = Number(resolveFromPath(step.key, { store: next }) ?? 0);
      const amount = Number(step.amount ?? 1);

      setByPath(next, step.key, current + amount);
      return next;
    }

    case "decrement": {
      if (!step.key) return next;

      const current = Number(resolveFromPath(step.key, { store: next }) ?? 0);
      const amount = Number(step.amount ?? 1);

      setByPath(next, step.key, current - amount);
      return next;
    }

    case "toggle": {
      if (!step.key) return next;

      const current = !!resolveFromPath(step.key, { store: next });
      setByPath(next, step.key, !current);
      return next;
    }

    case "execute_function": {
      return await CallPlgFnStep(step, next, callPluginFunction);
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
  const { callPluginFunction } = usePlugins();

  useEffect(() => {
    setStore(manifest?.store ?? {});
  }, [manifest]);

    async function handleAction(action: string, event?: any) {
        if (!manifest?.interactions?.length) return;

        const matching = manifest.interactions.filter(
            (interaction) => interaction.on === action
        );

        if (!matching.length) return;

        const prev = store; // current snapshot
        let next = { ...prev };

        for (const interaction of matching) {
            for (const step of interaction.steps ?? []) {
                next = await executeStep(next, step, callPluginFunction, event);
            }
        }

        setStore(next);
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