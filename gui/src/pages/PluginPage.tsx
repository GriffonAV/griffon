import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useParams } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import type { PluginManifest } from "@/bindings/PluginContext";
import { debug } from "@tauri-apps/plugin-log";
import { PageTabsLayout } from "@/components/layout/PageTabsLayout";

interface PluginInfo {
  pid: number;
  name: string;
  functions: string[];
  manifest?: PluginManifest; // Add manifest as optional
}

export default function PluginPage() {
  const { pid } = useParams();
  const [plugin, setPlugin] = useState<PluginInfo | null>(null);
  const [logs, setLogs] = useState<string[]>([]);

  useEffect(() => {
    // Load plugin info
    invoke<PluginInfo[]>("list_plugins_cmd").then((list) => {
      const found = list.find((p) => p.pid.toString() === pid);
      if (found) {
        setPlugin(found);
        // Load manifest for this plugin
        invoke<PluginManifest>("get_plugin_manifest", { pid: Number(pid) })
          .then((manifest) => {
            debug(JSON.stringify(manifest));
            setPlugin((prev) => (prev ? { ...prev, manifest } : null));
          })
          .catch((err) => {
            debug("Failed to load manifest:" + err);
            console.error("Failed to load manifest:", err);
          });
      }
    });

    // Listen for logs
    const unlisten = listen<string>("plugin-log", (event) => {
      setLogs((prev) => [...prev, event.payload]);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, [pid]);


  function send(msg: string) {
    if (!plugin) return;
    invoke("message_plugin", { pid: plugin.pid, msg });
  }

  if (!plugin) return <div>Plugin not found</div>;

  return (
    <PageTabsLayout title={plugin.name} navigation={plugin.manifest?.plugin?.tabs ? true : false} tabs={plugin.manifest?.plugin?.tabs}>
      <div className="flex flex-col h-full gap-4">
        <h1 className="text-lg font-semibold">
          {plugin.name} (PID {plugin.pid})
        </h1>

        <div className="flex gap-2">
          {plugin.functions.map((fn) => (
            <Button className="cursor-pointer" key={fn} onClick={() => send(fn)}>
              {fn}
            </Button>
          ))}
        </div>

        <Card className="flex-1 p-3 bg-black text-green-400 font-mono text-sm overflow-auto border">
          {logs.length === 0 ? (
            <span className="opacity-50">No output yet…</span>
          ) : (
            logs.map((line, i) => (
              <div key={i} className="whitespace-pre-wrap">
                $ {line}
              </div>
            ))
          )}
        </Card>
      </div>
      <div className="flex flex-col h-full gap-4">

      </div>
    </PageTabsLayout>
  );
}
