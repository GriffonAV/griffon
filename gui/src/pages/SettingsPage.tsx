import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ChangeThemeButtonTest, ModeToggleGroup } from "@/components/layout/ModeToggle";
import { PageLayout } from "@/components/layout/PageLayout";
import { PluginToggleSettings } from "@/components/layout/PluginToggleSettings";
import { PluginInstaller } from "@/components/layout/PluginInstaller";
import { usePlugins } from "@/bindings/PluginContext";

const PLUGIN_DOC_URL = "https://griffon-av.vercel.app/";

type SettingsTab = "Appearance" | "Notifications" | "Plugins";

const tabs: SettingsTab[] = ["Appearance", "Notifications", "Plugins"];

export default function SettingsPage() {
    const [activeTab, setActiveTab] = useState<SettingsTab>("Appearance");
    const [pluginBeingDeleted, setPluginBeingDeleted] = useState<string | null>(null);
    const [pluginRefreshKey, setPluginRefreshKey] = useState(0);
    const [isRefreshingPlugins, setIsRefreshingPlugins] = useState(false);

    const { plugins, refreshPlugins } = usePlugins();

    const refreshPluginUi = async () => {
        try {
            setIsRefreshingPlugins(true);

            await refreshPlugins();

            setPluginRefreshKey((key) => key + 1);
        } finally {
            setIsRefreshingPlugins(false);
        }
    };

    const handleManualRefresh = async () => {
        try {
            await refreshPluginUi();
        } catch (err) {
            console.error(err);
            alert("Failed to refresh plugins.");
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
                            onClick={() => setActiveTab(tab)}
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

                {activeTab === "Notifications" && (
                    <section className="rounded-xl border border-border bg-card p-6 shadow-sm">
                        <h2 className="text-xl font-bold">Notifications</h2>

                        <p className="mt-3 text-sm leading-6 text-muted-foreground">
                            Configure how Griffon should notify you about scans, alerts, plugin
                            activity, and security events.
                        </p>
                    </section>
                )}

                {activeTab === "Plugins" && (
                    <section className="rounded-xl border border-border bg-card p-6 shadow-sm">
                        <div className="flex flex-col gap-6">
                            <div className="flex flex-col gap-5 sm:flex-row sm:items-center sm:justify-between">
                                <div>
                                    <h2 className="text-xl font-bold">Plugins</h2>

                                    <p className="mt-2 max-w-xl text-sm leading-6 text-muted-foreground">
                                        Add, enable, disable or delete installed plugins, and access
                                        the plugin development documentation.
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
                                    Select a plugin manifest file and its compiled shared library.
                                    Griffon will copy them into <code>.config/griffon</code>.
                                </p>

                                <div className="mt-4">
                                    <PluginInstaller onInstalled={refreshPluginUi} />
                                </div>
                            </div>

                            <div className="border-t border-border pt-5">
                                <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                                    <div>
                                        <h3 className="text-lg font-semibold">Installed plugins</h3>

                                        <p className="mt-1 text-sm text-muted-foreground">
                                            Toggle plugin status directly from the settings panel.
                                        </p>
                                    </div>

                                    <button
                                        type="button"
                                        onClick={handleManualRefresh}
                                        disabled={isRefreshingPlugins}
                                        className="inline-flex items-center justify-center rounded-lg bg-secondary px-4 py-2 text-sm font-semibold text-secondary-foreground shadow-sm transition hover:opacity-90 active:scale-95 disabled:cursor-not-allowed disabled:opacity-60"
                                    >
                                        {isRefreshingPlugins ? "Refreshing..." : "Refresh"}
                                    </button>
                                </div>

                                <div className="mt-4">
                                    <PluginToggleSettings key={pluginRefreshKey} />
                                </div>
                            </div>

                            <div className="border-t border-border pt-5">
                                <h3 className="text-lg font-semibold text-red-600">
                                    Delete plugins
                                </h3>

                                <p className="mt-1 text-sm text-muted-foreground">
                                    Permanently remove a plugin from Griffon. This action will delete
                                    its manifest and shared library files.
                                </p>

                                {plugins.length === 0 ? (
                                    <p className="mt-4 text-sm text-muted-foreground">
                                        No installed plugin found.
                                    </p>
                                ) : (
                                    <div className="mt-4 space-y-3">
                                        {plugins.map((plugin) => (
                                            <div
                                                key={plugin.uuid}
                                                className="flex flex-col gap-4 rounded-lg border border-border p-4 sm:flex-row sm:items-center sm:justify-between"
                                            >
                                                <div>
                                                    <p className="font-semibold">
                                                        {plugin.display_name}
                                                    </p>

                                                    <p className="mt-1 text-sm text-muted-foreground">
                                                        {plugin.description || "No description available."}
                                                    </p>

                                                    <p className="mt-1 text-xs text-muted-foreground">
                                                        {plugin.file_name} • Version {plugin.version} •{" "}
                                                        {plugin.author}
                                                    </p>
                                                </div>

                                                <button
                                                    type="button"
                                                    disabled={pluginBeingDeleted === plugin.file_name}
                                                    onClick={() =>
                                                        handleDeletePlugin(
                                                            plugin.file_name,
                                                            plugin.display_name
                                                        )
                                                    }
                                                    className="inline-flex items-center justify-center rounded-lg bg-red-600 px-4 py-2 text-sm font-semibold text-white shadow-sm transition hover:bg-red-700 active:scale-95 disabled:cursor-not-allowed disabled:opacity-60"
                                                >
                                                    {pluginBeingDeleted === plugin.file_name
                                                        ? "Deleting..."
                                                        : "Delete"}
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