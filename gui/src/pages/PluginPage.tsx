import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useParams } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { PageWrapper } from "@/components/layout/PageLayout";

interface PluginInfo {
  pid: number;
  name: string;
  functions: string[];
}

export default function PluginPage() {
  const { pid } = useParams();
  const [plugin, setPlugin] = useState<PluginInfo | null>(null);
  const [logs, setLogs] = useState<string[]>([]);

  useEffect(() => {
    invoke<PluginInfo[]>("list_plugins_cmd").then((list) => {
      const found = list.find((p) => p.pid.toString() === pid);
      if (found) setPlugin(found);
    });

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
    <PageWrapper title={plugin.name}>
      <div className="flex flex-col h-full gap-4">
        <h1 className="text-lg font-semibold">
          {plugin.name} (PID {plugin.pid})
        </h1>

        <div className="flex gap-2">
          {plugin.functions.map((fn) => (
            <Button className="cursor-pointer" key={fn} onClick={() => send(fn)}>{fn}</Button>
          ))}
        </div>

        <Card className="flex-1 p-3 bg-black text-green-400 font-mono text-sm overflow-auto border">
          {logs.length === 0 ? (
            <span className="opacity-50">No output yet…</span>
          ) : (
            logs.map((line, i) => (
              <div key={i} className="whitespace-pre-wrap">$ {line}</div>
            ))
          )}
        </Card>
      </div>
    </PageWrapper>
  );
}
