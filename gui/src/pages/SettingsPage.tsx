import { useEffect, useState } from "react";
import { ChangeThemeButtonTest, ModeToggleGroup } from "@/components/layout/ModeToggle";
import { PageLayout } from "@/components/layout/PageLayout";
import { PluginToggleSettings } from "@/components/settings/PluginToggleSettings";
import { PluginInstaller } from "@/components/settings/PluginInstaller";
import { usePlugins } from "@/bindings/PluginContext";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { SquareArrowOutUpRight } from "lucide-react";
import { NotificationSettingsTable } from "@/components/settings/NotificationSettingsTable";
import { DeletePluginTable } from "@/components/settings/DeletePluginTable";
import { GlobalNotificationToggle } from "@/components/settings/GlobalNotificationToggle";
import { getVersion } from '@tauri-apps/api/app';

const PLUGIN_DOC_URL = "https://griffon-av.vercel.app/";

export default function SettingsPage() {
  const { refreshPlugins } = usePlugins();

  const refreshPluginUi = async () => {
    await refreshPlugins();
  };

  const [version, setVersion] = useState("");

    useEffect(() => {
      getVersion().then(setVersion);
    }, []);


  return (
    <PageLayout
      title="Settings"
      navigation={true}
      tabs={["Appearance", "Notifications", "Extensions", "About"]}
    >
      {/* --- APPEARANCE TAB --- */}
      <div title="Appearance" className="mt-2 w-full">
        <section className="rounded-md border border-border bg-card p-6 shadow-sm">
          <h2 className="text-lg font-semibold">Appearance</h2>
          <div className="mt-6 flex flex-col gap-4">
            <div className="flex flex-col gap-3">
              <span>Choose theme:</span>
              <ModeToggleGroup />
            </div>
            <ChangeThemeButtonTest />
          </div>
        </section>
      </div>

      <div title="Notifications" className="mt-2 w-full">
        <section className="rounded-md border border-border bg-card p-6 shadow-sm">
          <h2 className="text-lg font-semibold">Notifications</h2>

          <p className="mt-1 text-sm text-muted-foreground">
            Enable or disable notifications globally or for each installed extension.
          </p>

          <GlobalNotificationToggle onToggle={refreshPluginUi} />

          <NotificationSettingsTable />
        </section>
      </div>

      {/* --- EXTENSIONS TAB --- */}
      <div title="Extensions" className="mt-2 w-full">
        <section className="rounded-md border border-border bg-card p-6 shadow-sm">
          <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <h2 className="text-lg font-semibold">Extensions</h2>
                <p className="mt-2 max-w-xl text-sm leading-6 text-muted-foreground">
                  Add, enable, disable, delete extensions, and manage extension notifications.
                </p>
              </div>

              <Button asChild>
                <a
                  href={PLUGIN_DOC_URL}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center gap-2 cursor-pointer"
                >
                  Open documentation
                  <SquareArrowOutUpRight />
                </a>
              </Button>
            </div>

            <div className="border-t border-border pt-6">
              <h3 className="text-base font-semibold">Add Extension</h3>
              <p className="mt-1 text-sm text-muted-foreground">
                Install an extension by providing its shared library file. You can also find extensions in the{" "}
              </p>
              <div className="mt-4">
                <PluginInstaller onInstalled={refreshPluginUi} />
              </div>
            </div>

            <div className="border-t border-border pt-6">
              <div>
                <h3 className="text-base font-semibold">Installed extensions</h3>
                <p className="mt-1 text-sm text-muted-foreground">
                  Toggle extension status directly from the settings panel.
                </p>
              </div>
              <div className="mt-4">
                <PluginToggleSettings />
              </div>
            </div>

            <div className="border-t border-border pt-6">
              <h3 className="text-base font-semibold text-red-600">Delete extensions</h3>
              <p className="mt-1 text-sm text-muted-foreground">
                Permanently remove an extension from Griffon. This action will delete its manifest and
                shared library files.
              </p>

              {/* Replace the old mapping logic with the new component */}
              <DeletePluginTable/>
            </div>
          </div>
        </section>
      </div>

      <div title="About" className="mt-2 w-full">
        <section className="rounded-md border border-border bg-card p-6 shadow-sm">
          <h2 className="text-lg font-semibold mb-2">About Griffon</h2>
          <p className="text-sm text-muted-foreground flex items-center gap-2">
            You are using Griffon in version
            <Badge>{version}</Badge>.
          </p>
        </section>
      </div>
    </PageLayout>
  );
}
