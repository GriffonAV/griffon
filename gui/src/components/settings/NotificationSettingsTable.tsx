import { Button } from "@/components/ui/button";
import { usePlugins } from "@/bindings/PluginContext";

export function NotificationSettingsTable() {
    const { plugins, switchPluginNotificationStatus } = usePlugins();

    if (plugins.length === 0) {
        return <p className="mt-4 text-sm text-muted-foreground">No installed plugin found.</p>;
    }

    return (
        <div className="mt-4 w-full overflow-x-auto rounded-md border border-border">
            <table className="w-full text-left text-sm">
                <thead className="border-b border-border bg-muted/50 text-xs text-muted-foreground">
                    <tr>
                        <th className="px-4 py-3 font-medium">Plugin</th>
                        <th className="px-4 py-3 font-medium">Details</th>
                        <th className="px-4 py-3 font-medium">Status</th>
                        <th className="px-4 py-3 font-medium text-right">Action</th>
                    </tr>
                </thead>
                <tbody className="divide-y divide-border">
                    {plugins.map((plugin) => {
                        const isEnabled = plugin.notifications_enabled;
                        const isBusy = plugin.notification_status === "busy";

                        return <tr key={plugin.uuid} className="bg-card hover:bg-muted/20 transition-colors">
                            <td className="px-4 py-3 font-semibold">{plugin.display_name}</td>
                            <td className="px-4 py-3 text-muted-foreground text-xs">
                                {plugin.file_name} • v{plugin.version} • {plugin.author}
                            </td>
                            <td className="px-4 py-3">
                                <span
                                    className={
                                        isEnabled
                                            ? "text-green-600 font-medium"
                                            : "text-muted-foreground"
                                    }
                                >
                                    {isBusy ? "Enabled" : "Disabled"}
                                </span>
                            </td>
                            <td className="px-4 py-3 text-right">
                                <Button
                                    size="sm"
                                    variant={isEnabled ? "secondary" : "default"}
                                    disabled={isBusy}
                                    onClick={() => void switchPluginNotificationStatus(plugin.uuid)}
                                    className="cursor-pointer min-w-24"
                                >
                                    {isBusy
                                        ? "Switching..."
                                        : isEnabled
                                            ? "Disable"
                                            : "Enable"}
                                </Button>
                            </td>
                        </tr>;
                    })}
                </tbody>
            </table>
        </div>
    );
}
