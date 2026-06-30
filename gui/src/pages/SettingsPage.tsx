import { useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { ChangeThemeButtonTest, ModeToggleGroup } from "@/components/layout/ModeToggle";
import { PageLayout } from "@/components/layout/PageLayout";
import { PluginToggleSettings } from "@/components/layout/PluginToggleSettings";
import { PluginInstaller } from "@/components/layout/PluginInstaller";
import { usePlugins } from "@/bindings/PluginContext";

const PLUGIN_DOC_URL = "https://griffon-av.vercel.app/";

type SettingsTab = "Appearance" | "Plugins";

const tabs: SettingsTab[] = ["Appearance", "Plugins"];

const getInitialTab = (tab: string | null): SettingsTab => {
  switch (tab?.toLowerCase()) {
    case "plugins":
      return "Plugins";
    case "appearance":
    default:
      return "Appearance";
  }
};

const createRequestId = () => {
  return Math.floor(Date.now() % 1_000_000_000);
};

export default function SettingsPage() {
  const [searchParams, setSearchParams] = useSearchParams();

  const [activeTab, setActiveTab] = useState<SettingsTab>(() =>
    getInitialTab(searchParams.get("tab"))
  );

  const [pluginBeingDeleted, setPluginBeingDeleted] = useState<string | null>(null);
  const [pluginNotificationBeingSwitched, setPluginNotificationBeingSwitched] = useState<
    string | null
  >(null);
  const [pluginRefreshKey, setPluginRefreshKey] = useState(0);

  const { plugins, refreshPlugins } = usePlugins();

  useEffect(() => {
    setActiveTab(getInitialTab(searchParams.get("tab")));
  }, [searchParams]);

  const handleTabChange = (tab: SettingsTab) => {
    setActiveTab(tab);

    const nextParams = new URLSearchParams(searchParams);
    nextParams.set("tab", tab.toLowerCase());

    setSearchParams(nextParams);
  };

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

  const handleDeletePlugin = async (pluginFileName: string, pluginDisplayName: string) => {
    const confirmed = window.confirm(
      `Are you sure you want to delete "${pluginDisplayName}"?\n\nThis will remove the plugin .toml and .so files.`
    );

    if (!confirmed) return;

    try {
      setPluginBeingDeleted(pluginFileName);

      await invoke("delete_plugin", {
        name: pluginFileName,
      });

      await refreshPluginUi();
    } catch (err) {
      console.error(err);
      alert("Failed to delete plugin.");
    } finally {
      setPluginBeingDeleted(null);
    }
  };

  return (
    <PageLayout title="Settings" navigation={true}>
      <div className="space-y-6">
        <div className="flex gap-2 border-b border-border pb-2">
          {tabs.map((tab) => (
            <button
              key={tab}
              type="button"
              onClick={() => handleTabChange(tab)}
              className={`rounded-md px-4 py-2 text-sm font-medium transition ${
                activeTab === tab
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:bg-muted hover:text-foreground"
              }`}
            >
              {tab}
            </button>
          ))}
        </div>

        {activeTab === "Appearance" && (
          <section className="rounded-xl border border-border bg-card p-6 shadow-sm">
            <h2 className="text-xl font-bold">Appearance</h2>

            <div className="mt-5 flex flex-col gap-4">
              <div className="flex flex-col gap-2 sm:flex-row sm:items-center">
                <span className="text-sm font-medium">Choose theme:</span>
                <ModeToggleGroup />
              </div>

              <ChangeThemeButtonTest />
            </div>
          </section>
        )}

        {activeTab === "Plugins" && (
          <section className="rounded-xl border border-border bg-card p-6 shadow-sm">
            <div className="flex flex-col gap-6">
              <div className="flex flex-col gap-5 sm:flex-row sm:items-center sm:justify-between">
                <div>
                  <h2 className="text-xl font-bold">Plugins</h2>

                  <p className="mt-2 max-w-xl text-sm leading-6 text-muted-foreground">
                    Add, enable, disable, delete plugins, and manage plugin notifications.
                  </p>
                </div>

                <a
                  href={PLUGIN_DOC_URL}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center justify-center gap-2 rounded-lg bg-primary px-5 py-2.5 text-sm font-semibold text-primary-foreground shadow-sm transition hover:opacity-90 active:scale-95"
                >
                  Open documentation
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-4 w-4"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    aria-hidden="true"
                  >
                    <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
                    <polyline points="15 3 21 3 21 9" />
                    <line x1="10" y1="14" x2="21" y2="3" />
                  </svg>
                </a>
              </div>

              <div className="border-t border-border pt-5">
                <h3 className="text-lg font-semibold">Add plugin</h3>

                <p className="mt-1 text-sm text-muted-foreground">
                  Select a plugin manifest file and its compiled shared library. Griffon will copy
                  them into <code>.config/griffon</code>.
                </p>

                <div className="mt-4">
                  <PluginInstaller onInstalled={refreshPluginUi} />
                </div>
              </div>

              <div className="border-t border-border pt-5">
                <div>
                  <h3 className="text-lg font-semibold">Installed plugins</h3>

                  <p className="mt-1 text-sm text-muted-foreground">
                    Toggle plugin status directly from the settings panel.
                  </p>
                </div>

                <div className="mt-4">
                  <PluginToggleSettings key={pluginRefreshKey} />
                </div>
              </div>

              <div className="border-t border-border pt-5">
                <h3 className="text-lg font-semibold">Plugin notifications</h3>

                <p className="mt-1 text-sm text-muted-foreground">
                  Enable or disable notifications for each installed plugin.
                </p>

                {plugins.length === 0 ? (
                  <p className="mt-4 text-sm text-muted-foreground">No installed plugin found.</p>
                ) : (
                  <div className="mt-4 space-y-3">
                    {plugins.map((plugin) => (
                      <div
                        key={plugin.uuid}
                        className="flex flex-col gap-4 rounded-lg border border-border p-4 sm:flex-row sm:items-center sm:justify-between"
                      >
                        <div>
                          <p className="font-semibold">{plugin.display_name}</p>

                          <p className="mt-1 text-sm text-muted-foreground">
                            {plugin.description || "No description available."}
                          </p>

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

                        <button
                          type="button"
                          disabled={pluginNotificationBeingSwitched === plugin.uuid}
                          onClick={() => handleSwitchNotification(plugin.uuid, plugin.display_name)}
                          className={`inline-flex items-center justify-center rounded-lg px-4 py-2 text-sm font-semibold shadow-sm transition active:scale-95 disabled:cursor-not-allowed disabled:opacity-60 ${
                            plugin.notifications_enabled
                              ? "bg-secondary text-secondary-foreground hover:opacity-90"
                              : "bg-primary text-primary-foreground hover:opacity-90"
                          }`}
                        >
                          {pluginNotificationBeingSwitched === plugin.uuid
                            ? "Switching..."
                            : plugin.notifications_enabled
                              ? "Disable notifications"
                              : "Enable notifications"}
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              <div className="border-t border-border pt-5">
                <h3 className="text-lg font-semibold text-red-600">Delete plugins</h3>

                <p className="mt-1 text-sm text-muted-foreground">
                  Permanently remove a plugin from Griffon. This action will delete its manifest and
                  shared library files.
                </p>

                {plugins.length === 0 ? (
                  <p className="mt-4 text-sm text-muted-foreground">No installed plugin found.</p>
                ) : (
                  <div className="mt-4 space-y-3">
                    {plugins.map((plugin) => (
                      <div
                        key={plugin.uuid}
                        className="flex flex-col gap-4 rounded-lg border border-border p-4 sm:flex-row sm:items-center sm:justify-between"
                      >
                        <div>
                          <p className="font-semibold">{plugin.display_name}</p>

                          <p className="mt-1 text-sm text-muted-foreground">
                            {plugin.description || "No description available."}
                          </p>

                          <p className="mt-1 text-xs text-muted-foreground">
                            {plugin.file_name} • Version {plugin.version} • {plugin.author}
                          </p>
                        </div>

                        <button
                          type="button"
                          disabled={pluginBeingDeleted === plugin.file_name}
                          onClick={() => handleDeletePlugin(plugin.file_name, plugin.display_name)}
                          className="inline-flex items-center justify-center rounded-lg bg-red-600 px-4 py-2 text-sm font-semibold text-white shadow-sm transition hover:bg-red-700 active:scale-95 disabled:cursor-not-allowed disabled:opacity-60"
                        >
                          {pluginBeingDeleted === plugin.file_name ? "Deleting..." : "Delete"}
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </section>
        )}
      </div>
    </PageLayout>
  );
}
