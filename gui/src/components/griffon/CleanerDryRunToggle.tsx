import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { resolveFromPath } from "@/lib/utils";
import type { GriffonActionHandler } from "../types";

interface CleanerDryRunToggleProps {
    element: {
        type: "cleaner_dry_run_toggle";
        id: string;
        title?: string;
        description?: string;
        from?: string;
        action?: string;
    };
    store?: Record<string, any>;
    onAction?: GriffonActionHandler;
}

type CleanerOptions = {
    dry_run?: boolean;
    file_types?: string[];
};

export default function CleanerDryRunToggle({
                                                element,
                                                store = {},
                                                onAction,
                                            }: CleanerDryRunToggleProps) {
    const context = { store };

    const options: CleanerOptions = element.from
        ? resolveFromPath(element.from, context)
        : { dry_run: true, file_types: [] };

    const dryRun = options?.dry_run ?? true;

    const updateDryRun = (checked: boolean) => {
        if (!element.action || !onAction) {
            return;
        }

        onAction(element.action, {
            ...element,
            value: {
                ...options,
                dry_run: checked,
            },
        });
    };

    return (
        <div id={element.id} className="flex items-center justify-between rounded-md border bg-muted/20 p-4">
            <div className="space-y-1">
                <div className="flex items-center gap-2">
                    <p className="text-sm font-medium">
                        {element.title ?? "Dry-run mode"}
                    </p>

                    <Badge variant={dryRun ? "outline" : "destructive"}>
                        {dryRun ? "Safe preview" : "Real deletion"}
                    </Badge>
                </div>

                <p className="text-xs text-muted-foreground">
                    {element.description ??
                        "When enabled, Griffon only simulates the cleanup without deleting files."}
                </p>
            </div>

            <Switch checked={dryRun} onCheckedChange={updateDryRun} />
        </div>
    );
}