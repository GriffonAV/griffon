import { useEffect } from "react";
import { useParams } from "react-router-dom";
import { usePlugins } from "@/bindings/PluginContext";
import GriffonSectionRenderer from "@/renderer/GriffonSectionRenderer";
import type { GriffonSection } from "@/components/types";
import { useGriffonStore } from "@/hooks/useGriffonStore";
import { PageLayout } from "@/components/layout/PageLayout";

export default function PluginPage() {
  const { name } = useParams();
  const { currentManifest, loadPluginManifest, isManifestLoading } = usePlugins();
  const { store, handleAction } = useGriffonStore(currentManifest);

  useEffect(() => {
    if (name) {
      loadPluginManifest(name);
    }
  }, [name, loadPluginManifest]);


  if (isManifestLoading || !currentManifest?.plugin) {
    return <div>Loading extension...</div>;
  }

  return (
    <PageLayout mode="tabs" title={currentManifest.plugin.name} navigation tabs={currentManifest.plugin?.tabs}
    >

      {currentManifest.plugin?.tabs?.map((tab) => {
        const tabSections =
          currentManifest.ui?.sections?.filter(
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