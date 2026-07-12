import { Button } from "@/components/ui/button";

// Define the type based on the properties used in your SettingsPage
type Plugin = {
    uuid: string;
    display_name: string;
    file_name: string;
    version: string;
    author: string;
    notifications_enabled: boolean;
};

type NotificationSettingsTableProps = {
    plugins: Plugin[];
    switchingPluginUuid: string | null;
    onToggle: (uuid: string, displayName: string) => void;
};

export function NotificationSettingsTable({
    plugins,
    switchingPluginUuid,
    onToggle,
}: NotificationSettingsTableProps) {
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
                    {plugins.map((plugin) => (
                        <tr key={plugin.uuid} className="bg-card hover:bg-muted/20 transition-colors">
                            <td className="px-4 py-3 font-semibold">{plugin.display_name}</td>
                            <td className="px-4 py-3 text-muted-foreground text-xs">
                                {plugin.file_name} • v{plugin.version} • {plugin.author}
                            </td>
                            <td className="px-4 py-3">
                                <span
                                    className={
                                        plugin.notifications_enabled
                                            ? "text-green-600 font-medium"
                                            : "text-muted-foreground"
                                    }
                                >
                                    {plugin.notifications_enabled ? "Enabled" : "Disabled"}
                                </span>
                            </td>
                            <td className="px-4 py-3 text-right">
                                <Button
                                    size="sm"
                                    variant={plugin.notifications_enabled ? "secondary" : "default"}
                                    disabled={switchingPluginUuid === plugin.uuid}
                                    onClick={() => onToggle(plugin.uuid, plugin.display_name)}
                                    className="cursor-pointer min-w-24"
                                >
                                    {switchingPluginUuid === plugin.uuid
                                        ? "Switching..."
                                        : plugin.notifications_enabled
                                            ? "Disable"
                                            : "Enable"}
                                </Button>
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}