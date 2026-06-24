import { useState } from "react";
import { ChangeThemeButtonTest, ModeToggleGroup } from "@/components/layout/ModeToggle";
import { PageLayout } from "@/components/layout/PageLayout";
import { PluginToggleSettings } from "@/components/layout/PluginToggleSettings";
import { PluginInstaller } from "@/components/layout/PluginInstaller";

const PLUGIN_DOC_URL = "https://griffon-av.vercel.app/";

type SettingsTab = "Appearance" | "Notifications" | "Plugins";

const tabs: SettingsTab[] = ["Appearance", "Notifications", "Plugins"];

export default function SettingsPage() {
    const [activeTab, setActiveTab] = useState<SettingsTab>("Appearance");

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
                                        Add, enable or disable installed plugins, and access the plugin
                                        development documentation.
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
                                    <PluginInstaller />
                                </div>
                            </div>

                            <div className="border-t border-border pt-5">
                                <h3 className="text-lg font-semibold">Installed plugins</h3>

                                <p className="mt-1 text-sm text-muted-foreground">
                                    Toggle plugin status directly from the settings panel.
                                </p>

                                <div className="mt-4">
                                    <PluginToggleSettings />
                                </div>
                            </div>
                        </div>
                    </section>
                )}
            </div>
        </PageLayout>
    );
}