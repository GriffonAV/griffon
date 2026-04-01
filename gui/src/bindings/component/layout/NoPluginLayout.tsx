import { Button } from "@/components/ui/button";
import { usePlugins } from "../../PluginContext";
import { openPath } from "@tauri-apps/plugin-opener";
import type { ReactNode } from "react";

export function NoPluginLayout({ children }: { children: ReactNode }) {
    const { plugins, isLoading } = usePlugins();

    if (isLoading) {
        return <div>Loading plugins...</div>;
    }

    const openfolder = () => {
        openPath("~/Downloads").catch((err) => {
            alert("Failed to open folder:" + err);
        }).finally(() => {
            alert("open folder:");
        });
    }

    if (plugins.length === 0) {
        return (
            <div className="">
                It looks like you don't have any plugins installed. Please install a plugin to continue.
                <br />
                <br />
                Add your plugin folder in <Button onClick={openfolder} variant={"ghost"}>~/.config/griffon/plugins/</Button> and refresh.
                < br />
                <br />
                <Button>Refresh plugins</Button>
            </div >

        );
    }

    return <>{children}</>;
}
