import { useEffect, useState, type JSX } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useParams } from "react-router-dom";
import type { PluginManifest } from "@/bindings/PluginContext";
import { debug } from "@tauri-apps/plugin-log";
import { PageTabsLayout } from "@/components/layout/PageTabsLayout";
import GriffonSectionRenderer from "@/renderer/GriffonSectionRenderer";
import type { GriffonSection } from "@/components/types";

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
    <PageTabsLayout
        title={plugin.name}
        navigation={!!plugin.manifest?.plugin?.tabs?.length}
        tabs={plugin.manifest?.plugin?.tabs}
    >
        {plugin.manifest?.plugin?.tabs?.map((tab) => {
        const tabSections =
            plugin.manifest?.ui?.sections?.filter((section) => section.tab === tab) ?? [];

        return (
            <div className="flex flex-col h-full gap-4" key={tab}>
            {tabSections.map((section) => (
                <div key={section.id} className="mx-5">
                <GriffonSectionRenderer section={section as GriffonSection} />
                </div>
            ))}
            </div>
        );
        })}
    </PageTabsLayout>
    );
}
