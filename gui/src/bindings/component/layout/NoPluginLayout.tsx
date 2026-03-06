import { usePlugins } from "../../PluginContext";

export function NoPluginLayout({ children }: { children: ReactNode }) {
    const { plugins, isLoading } = usePlugins();

    if (isLoading) {
        return <div>Loading plugins...</div>;
    }

    if (plugins.length === 0) {
        return (
            <div className="">
                It looks like you don't have any plugins installed. Please install a plugin to continue.

            </div>
        );
    }

    return <>{children}</>;
}
