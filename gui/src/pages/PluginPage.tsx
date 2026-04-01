import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useParams } from "react-router-dom";
import type { PluginManifest } from "@/bindings/PluginContext";
import { debug } from "@tauri-apps/plugin-log";
import { PageTabsLayout } from "@/components/layout/PageTabsLayout";
import GriffonSectionRenderer from "@/renderer/GriffonSectionRenderer";
import type { GriffonSection } from "@/components/types";
import { useGriffonStore } from "@/hooks/useGriffonStore";

interface PluginInfo {
  pid: number;
  name: string;
  manifest?: PluginManifest; // Add manifest as optional
}

export default function PluginPage() {
  const { name } = useParams();
  const [plugin, setPlugin] = useState<PluginInfo | null>(null);
  // @ts-ignore

  const [logs, setLogs] = useState<string[]>([]);
  const manifest = plugin?.manifest ?? null;
  const { store, handleAction } = useGriffonStore(manifest);

  useEffect(() => {

    setPlugin({ pid: -1, name: name ? name : "" }); // Set basic info first
    invoke<PluginManifest>("get_plugin_manifest", { name: name })
      .then((manifest) => {
        debug(JSON.stringify(manifest));
        setPlugin((prev) =>
          prev
            ? {
              ...prev,
              manifest: {
                ...manifest,
                store: manifest.store ?? {},
                interactions: manifest.interactions ?? [],
              },
            }
            : null
        );
      })
      .catch((err) => {
        debug("Failed to load manifest:" + err);
        console.error("Failed to load manifest:", err);
      });
    //   }
    // });

    // Listen for logs
    const unlisten = listen<string>("plugin-log", (event) => {
      setLogs((prev) => [...prev, event.payload]);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, [name]);


  // @ts-ignore
  // function send(msg: string) {
  //   if (!plugin) return;
  //   invoke("message_plugin", { pid: plugin.pid, msg });
  // }

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
                <GriffonSectionRenderer
                  section={section as GriffonSection}
                  store={store}
                  onAction={handleAction}
                />
              </div>
            ))}
          </div>
        );
      })}
    </PageTabsLayout>
  );
}