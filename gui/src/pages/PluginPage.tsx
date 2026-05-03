import { useEffect } from "react";
import { useParams } from "react-router-dom";
import { usePlugins } from "@/bindings/PluginContext";
import { PageTabsLayout } from "@/components/layout/PageTabsLayout";
import GriffonSectionRenderer from "@/renderer/GriffonSectionRenderer";
import type { GriffonSection } from "@/components/types";
import { useGriffonStore } from "@/hooks/useGriffonStore";

export default function PluginPage() {
  const { name } = useParams();
  const { currentManifest, loadPluginManifest, isManifestLoading } = usePlugins();
  const { store, handleAction } = useGriffonStore(currentManifest);

  useEffect(() => {
    if (name) {
      loadPluginManifest(name);
    }
  }, [name, loadPluginManifest]);


  if (isManifestLoading || !currentManifest) {
    return <div>Loading plugin...</div>;
  }

  return (
    <PageTabsLayout
      title={currentManifest.plugin.name}
      navigation={!!currentManifest.plugin?.tabs?.length}
      tabs={currentManifest.plugin?.tabs}
    >
      {currentManifest.plugin?.tabs?.map((tab) => {
        const tabSections =
          currentManifest.ui?.sections?.filter(
            (section) => section.tab === tab
          ) ?? [];

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