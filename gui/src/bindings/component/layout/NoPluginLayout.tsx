import { usePlugins } from "../../PluginContext";

export function NoPluginLayout({ children }: { children: ReactNode }) {
    const { plugins, isLoading } = usePlugins();

    if (isLoading) {
        return <div>Loading plugins...</div>;
    }

    if (plugins.length === 0) {
        return (
            <div className="flex items-center justify-center h-full">
                <div className="text-center p-8 bg-gray-100 rounded-lg shadow-md">
                    <h2 className="text-xl font-bold mb-4">No Plugins Found</h2>
                    <p className="text-gray-600">
                        It looks like you don't have any plugins installed. Please install a plugin to continue.
                    </p>
                </div>
            </div>
        );
    }

    return <>{children}</>;
}
