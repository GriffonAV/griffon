import { useEffect, useRef } from "react";
import { Link, useParams } from "react-router-dom";
import { usePlugins } from "@/bindings/PluginContext";
import GriffonSectionRenderer from "@/renderer/GriffonSectionRenderer";
import type { GriffonSection } from "@/components/types";
import { useGriffonStore } from "@/hooks/useGriffonStore";
import { PageLayout } from "@/components/layout/PageLayout";
import { Button } from "@/components/ui/button";

export default function PluginPage() {
  const { name } = useParams();
  const { plugins, currentPlugin, loadPlugin, isPluginLoading } = usePlugins();
  const { store, handleAction } = useGriffonStore(currentPlugin?.manifest || null);
  const requestedPlugin = plugins.find((plugin) => plugin.file_name === name);
  const lastLoadAttempt = useRef<string | null>(null);

  useEffect(() => {
    const loadAttempt = `${name}:${requestedPlugin?.status}`;

    if (
      name &&
      requestedPlugin?.status === "healthy" &&
      !isPluginLoading &&
      currentPlugin?.file_name !== name &&
      lastLoadAttempt.current !== loadAttempt
    ) {
      lastLoadAttempt.current = loadAttempt;
      loadPlugin(name);
    }
  }, [name, requestedPlugin?.status, currentPlugin?.file_name, isPluginLoading, loadPlugin]);


  if (requestedPlugin && requestedPlugin.status !== "healthy") {
    return (
      <PageLayout title={requestedPlugin.display_name}>
        <div className="m-5 rounded-md border border-border bg-muted/30 p-6">
          <h2 className="text-lg font-semibold">Extension unavailable</h2>
          <p className="mt-2 text-sm text-muted-foreground">
            Enable this extension before opening or using it.
          </p>
          <Button asChild className="mt-4">
            <Link to="/settings">Open extension settings</Link>
          </Button>
        </div>
      </PageLayout>
    );
  }

  if (isPluginLoading || !currentPlugin || currentPlugin.file_name !== name) {
    return <div>Loading extension...</div>;
  }

  const manifest = currentPlugin.manifest;

  return (
    <PageLayout mode="tabs" title={currentPlugin.display_name} navigation tabs={manifest?.plugin.tabs}
    >

      {manifest?.plugin?.tabs?.map((tab) => {
        const tabSections =
          manifest.ui?.sections?.filter(
            (section) => section.tab === tab
          ) ?? [];

        return (
          <div className="flex flex-col h-full w-full gap-4" key={tab}>
            {tabSections.map((section, index) => (
              <div key={section.id} className="mx-5">
                <GriffonSectionRenderer
                  section={section as GriffonSection}
                  store={store}
                  onAction={handleAction}
                />
                {index < tabSections.length - 1 && (
                  <hr className="my-4 border-border" />
                )}
              </div>
            ))}
          </div>
        );
      })}
    </PageLayout>
  );
}
