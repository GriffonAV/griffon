import { Button } from "@/components/ui/button";
import { usePlugins } from "../../PluginContext";
import type { ReactNode } from "react";
import { Link } from "react-router-dom";

export function NoPluginLayout({ children }: { children: ReactNode }) {
  const { plugins } = usePlugins();

  if (plugins.length === 0) {
    return (
      <div className="flex flex-col items-start gap-4 p-6 text-sm text-muted-foreground">
        <div>
          <p className="font-medium text-foreground">No plugins installed</p>

          <p className="mt-2">
            It looks like you don't have any plugins installed yet. Go to the plugin settings page
            to add one.
          </p>
        </div>

        <Link to="/settings?tab=plugins">
          <Button variant="default">Open plugin settings</Button>
        </Link>
      </div>
    );
  }

  return <>{children}</>;
}
