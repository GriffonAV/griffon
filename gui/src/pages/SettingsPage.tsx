import { useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { ChangeThemeButtonTest, ModeToggleGroup } from "@/components/layout/ModeToggle";
import { PageLayout } from "@/components/layout/PageLayout";
import { PluginToggleSettings } from "@/components/layout/PluginToggleSettings";
import { PluginInstaller } from "@/components/layout/PluginInstaller";
import { usePlugins } from "@/bindings/PluginContext";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { SquareArrowOutUpRight } from "lucide-react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog"; // <-- Added Shadcn Alert Dialog

const PLUGIN_DOC_URL = "https://griffon-av.vercel.app/";

const createRequestId = () => {
  return Math.floor(Date.now() % 1_000_000_000);
};

export default function SettingsPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const { plugins, refreshPlugins } = usePlugins();

  const [pluginBeingDeleted, setPluginBeingDeleted] = useState<string | null>(null);
  const [pluginNotificationBeingSwitched, setPluginNotificationBeingSwitched] = useState<string | null>(null);
  const [pluginRefreshKey, setPluginRefreshKey] = useState(0);

  const refreshPluginUi = async () => {
    await invoke("refresh_plugin");
    await new Promise((resolve) => setTimeout(resolve, 500));
    await refreshPlugins();
    setPluginRefreshKey((key) => key + 1);
  };

  const handleSwitchNotification = async (pluginUuid: string, pluginDisplayName: string) => {
    try {
      setPluginNotificationBeingSwitched(pluginUuid);
      await invoke("switch_status_notification", {
        pluginUuid,
        requestId: createRequestId(),
      });
      await new Promise((resolve) => setTimeout(resolve, 300));
      await refreshPluginUi();
    } catch (err) {
      console.error(err);
      alert(`Failed to switch notifications for "${pluginDisplayName}".`);
    } finally {
      setPluginNotificationBeingSwitched(null);
    }
  };

  // Removed window.confirm, logic is now triggered securely by the AlertDialog
  const handleDeletePlugin = async (pluginFileName: string) => {
    try {
      setPluginBeingDeleted(pluginFileName);
      await invoke("delete_plugin", { name: pluginFileName });
      await refreshPluginUi();
    } catch (err) {
      console.error(err);
      alert("Failed to delete plugin.");
    } finally {
      setPluginBeingDeleted(null);
    }
  };

  return (
    <PageLayout
      title="Settings"
      navigation={true}
      tabs={["Appearance", "Notifications", "Plugins", "About"]}
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
            Enable or disable notifications for each installed plugin.
          </p>

          {plugins.length === 0 ? (
            <p className="mt-4 text-sm text-muted-foreground">No installed plugin found.</p>
          ) : (
            <div className="mt-4 flex flex-col gap-4">
              {plugins.map((plugin) => (
                <div
                  key={plugin.uuid}
                  className="flex flex-col gap-4 rounded-md border border-border p-4 sm:flex-row sm:items-center sm:justify-between"
                >
                  <div>
                    <p className="text-base font-semibold">{plugin.display_name}</p>
                    <p className="mt-1 text-xs text-muted-foreground">
                      {plugin.file_name} • Version {plugin.version} • {plugin.author}
                    </p>
                    <p className="mt-2 text-xs font-medium">
                      Notifications:{" "}
                      <span
                        className={
                          plugin.notifications_enabled
                            ? "text-green-600"
                            : "text-muted-foreground"
                        }
                      >
                        {plugin.notifications_enabled ? "Enabled" : "Disabled"}
                      </span>
                    </p>
                  </div>

                  <Button
                    variant={plugin.notifications_enabled ? "secondary" : "default"}
                    disabled={pluginNotificationBeingSwitched === plugin.uuid}
                    onClick={() => handleSwitchNotification(plugin.uuid, plugin.display_name)}
                    className="cursor-pointer"
                  >
                    {pluginNotificationBeingSwitched === plugin.uuid
                      ? "Switching..."
                      : plugin.notifications_enabled
                        ? "Disable notifications"
                        : "Enable notifications"}
                  </Button>
                </div>
              ))}
            </div>
          )}
        </section>
      </div>

      {/* --- PLUGINS TAB --- */}
      <div title="Plugins" className="mt-2 w-full">
        <section className="rounded-md border border-border bg-card p-6 shadow-sm">
          <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <h2 className="text-lg font-semibold">Plugins</h2>
                <p className="mt-2 max-w-xl text-sm leading-6 text-muted-foreground">
                  Add, enable, disable, delete plugins, and manage plugin notifications.
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
              <h3 className="text-base font-semibold">Add plugin</h3>
              <p className="mt-1 text-sm text-muted-foreground">
                Select a plugin manifest file and its compiled shared library. Griffon will copy
                them into <code>.config/griffon</code>.
              </p>
              <div className="mt-4">
                <PluginInstaller onInstalled={refreshPluginUi} />
              </div>
            </div>

            <div className="border-t border-border pt-6">
              <div>
                <h3 className="text-base font-semibold">Installed plugins</h3>
                <p className="mt-1 text-sm text-muted-foreground">
                  Toggle plugin status directly from the settings panel.
                </p>
              </div>
              <div className="mt-4">
                <PluginToggleSettings key={pluginRefreshKey} />
              </div>
            </div>

            <div className="border-t border-border pt-6">
              <h3 className="text-base font-semibold text-red-600">Delete plugins</h3>
              <p className="mt-1 text-sm text-muted-foreground">
                Permanently remove a plugin from Griffon. This action will delete its manifest and
                shared library files.
              </p>

              {plugins.length === 0 ? (
                <p className="mt-4 text-sm text-muted-foreground">No installed plugin found.</p>
              ) : (
                /* Added max-h-64 (approx 16rem) and overflow-y-auto for scrolling, plus pr-2 so the scrollbar doesn't hug the border */
                <div className="mt-4 flex flex-col gap-2 max-h-64 overflow-y-auto pr-2">
                  {plugins.map((plugin) => (
                    <div
                      key={plugin.uuid}
                      /* Reduced padding to p-3 and forced single row alignment for a compacter list */
                      className="flex items-center justify-between rounded-md border border-border p-3"
                    >
                      {/* Stripped out extra info to make the list smaller */}
                      <p className="text-sm font-semibold">{plugin.display_name}</p>

                      <AlertDialog>
                        <AlertDialogTrigger asChild>
                          <Button
                            variant="destructive"
                            size="sm" /* Reduced button size for the compact row */
                            disabled={pluginBeingDeleted === plugin.file_name}
                            className="cursor-pointer"
                          >
                            {pluginBeingDeleted === plugin.file_name ? "Deleting..." : "Delete"}
                          </Button>
                        </AlertDialogTrigger>

                        <AlertDialogContent>
                          <AlertDialogHeader>
                            <AlertDialogTitle>Delete {plugin.display_name}?</AlertDialogTitle>
                            {/* Changed to asChild so we can render a custom div structure inside the description without HTML validation errors */}
                            <AlertDialogDescription asChild>
                              <div className="flex flex-col gap-3 mt-2 text-sm text-muted-foreground">
                                <p>
                                  Are you sure you want to delete this plugin? This action cannot be undone and will permanently remove the plugin files.
                                </p>

                                {/* Moved detailed information here into a visual "card" for review */}
                                <div className="bg-muted p-3 rounded-md flex flex-col gap-1 text-left text-xs">
                                  <p><strong className="text-foreground font-medium">Description:</strong> {plugin.description || "None"}</p>
                                  <p><strong className="text-foreground font-medium">File:</strong> {plugin.file_name}</p>
                                  <p><strong className="text-foreground font-medium">Version:</strong> {plugin.version}</p>
                                  <p><strong className="text-foreground font-medium">Author:</strong> {plugin.author}</p>
                                </div>
                              </div>
                            </AlertDialogDescription>
                          </AlertDialogHeader>

                          <AlertDialogFooter>
                            <AlertDialogCancel>Cancel</AlertDialogCancel>
                            <AlertDialogAction
                              onClick={() => handleDeletePlugin(plugin.file_name)}
                              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                            >
                              Delete
                            </AlertDialogAction>
                          </AlertDialogFooter>
                        </AlertDialogContent>
                      </AlertDialog>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </section>
      </div>

      <div title="About" className="mt-2 w-full">
        <section className="rounded-md border border-border bg-card p-6 shadow-sm">
          <h2 className="text-lg font-semibold mb-2">About Griffon</h2>
          <p className="text-sm text-muted-foreground flex items-center gap-2">
            You are using Griffon in version
            <Badge>0.3.0-alpha</Badge>.
          </p>
        </section>
      </div>
    </PageLayout>
  );
}