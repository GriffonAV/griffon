import {
    AlertTriangle,
    Box,
    CheckCircle2,
    Clock,
    Database,
    FileWarning,
    HardDrive,
    Info,
    ShieldCheck,
    Trash2,
} from "lucide-react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "@/components/ui/card";
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";

import { resolveFromPath } from "@/lib/utils";

interface CleanerRunOverviewProps {
    element: {
        id: string;
        title?: string;
        from?: string;
    };
    store?: Record<string, any>;
}

type CleanerRunResult = {
    summary?: CleanerSummary;
    modules?: CleanerModule[];
    selected_scope?: CleanerSelectedScope;
    metadata?: CleanerMetadata;
};

type CleanerSummary = {
    dry_run: boolean;
    total_reclaimable_bytes: number;
    total_files_touched: number;
    total_warnings: number;
    total_errors: number;
    total_permission_denied: number;
    duration_ms: number;
};

type CleanerModule = {
    id: string;
    label: string;
    reclaimable_bytes: number;
    files_touched: number;
    duration_ms: number;
    warnings_count: number;
    errors_count: number;
    permission_denied: number;
    candidate_files_count: number;
    deleted_files_count: number;
    skipped_files_count: number;
    warnings_preview: string[];
    warnings_truncated: boolean;
    errors_preview: string[];
    errors_truncated: boolean;
    top_root_paths: CleanerStatItem[];
    top_file_types: CleanerStatItem[];
    actions?: CleanerDockerAction[];
};

type CleanerStatItem = {
    name: string;
    files_touched: number;
    bytes: number;
};

type CleanerDockerAction = {
    name: string;
    command: string;
    enabled: boolean;
    status: string;
    reason: string;
};

type CleanerSelectedScope = {
    profile?: string;
    enabled_categories?: string[];
    selected_file_types?: string[];
    dry_run?: boolean;
};

type CleanerMetadata = {
    run_id?: string;
    generated_at?: string;
    plugin_name?: string;
    plugin_version?: string;
};

function formatBytes(bytes?: number): string {
    if (!bytes || bytes <= 0) {
        return "0 B";
    }

    const units = ["B", "KB", "MB", "GB", "TB"];
    const index = Math.floor(Math.log(bytes) / Math.log(1000));
    const value = bytes / Math.pow(1000, index);

    return `${value.toFixed(value >= 10 ? 1 : 2)} ${units[index]}`;
}

function formatDate(value?: string): string {
    if (!value) {
        return "No run yet";
    }

    const date = new Date(value);

    if (Number.isNaN(date.getTime())) {
        return value;
    }

    return date.toLocaleString();
}

function hasRunResult(result?: CleanerRunResult): boolean {
    return Boolean(
        result?.summary &&
        typeof result.summary.total_reclaimable_bytes === "number"
    );
}

function getModuleBadgeVariant(module: CleanerModule) {
    if (module.errors_count > 0) {
        return "destructive" as const;
    }

    if (module.warnings_count > 0 || module.permission_denied > 0) {
        return "secondary" as const;
    }

    return "default" as const;
}

function getAllWarnings(modules: CleanerModule[]): string[] {
    return modules.flatMap((module) =>
        (module.warnings_preview ?? []).map(
            (warning) => `${module.label}: ${warning}`
        )
    );
}

function SummaryCard({
                         title,
                         value,
                         description,
                         icon,
                     }: {
    title: string;
    value: string;
    description: string;
    icon: React.ReactNode;
}) {
    return (
        <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">
                    {title}
                </CardTitle>
                <div className="text-muted-foreground">{icon}</div>
            </CardHeader>

            <CardContent>
                <p className="text-2xl font-bold">{value}</p>
                <p className="text-xs text-muted-foreground">
                    {description}
                </p>
            </CardContent>
        </Card>
    );
}

function ModuleCard({ module }: { module: CleanerModule }) {
    const dockerActions = module.actions ?? [];
    const enabledActions = dockerActions.filter((action) => action.enabled);
    const disabledActions = dockerActions.filter((action) => !action.enabled);

    return (
        <Card>
            <CardHeader>
                <div className="flex items-start justify-between gap-4">
                    <div>
                        <CardTitle className="flex items-center gap-2">
                            {module.id === "docker" ? (
                                <Box className="h-5 w-5 text-muted-foreground" />
                            ) : (
                                <HardDrive className="h-5 w-5 text-muted-foreground" />
                            )}
                            {module.label}
                        </CardTitle>

                        <CardDescription>
                            {module.files_touched.toLocaleString()} file(s) touched ·{" "}
                            {module.duration_ms} ms
                        </CardDescription>
                    </div>

                    <Badge variant={getModuleBadgeVariant(module)}>
                        {module.errors_count > 0
                            ? "Error"
                            : module.warnings_count > 0
                                ? "Warning"
                                : "OK"}
                    </Badge>
                </div>
            </CardHeader>

            <CardContent className="space-y-4">
                <div>
                    <p className="text-2xl font-bold">
                        {formatBytes(module.reclaimable_bytes)}
                    </p>
                    <p className="text-sm text-muted-foreground">
                        Reclaimable space
                    </p>
                </div>

                <div className="grid grid-cols-2 gap-3 text-sm">
                    <div className="rounded-md border bg-muted/30 p-3">
                        <p className="font-semibold">
                            {module.candidate_files_count.toLocaleString()}
                        </p>
                        <p className="text-muted-foreground">Candidates</p>
                    </div>

                    <div className="rounded-md border bg-muted/30 p-3">
                        <p className="font-semibold">
                            {module.skipped_files_count.toLocaleString()}
                        </p>
                        <p className="text-muted-foreground">Skipped</p>
                    </div>

                    <div className="rounded-md border bg-muted/30 p-3">
                        <p className="font-semibold">{module.warnings_count}</p>
                        <p className="text-muted-foreground">Warnings</p>
                    </div>

                    <div className="rounded-md border bg-muted/30 p-3">
                        <p className="font-semibold">{module.permission_denied}</p>
                        <p className="text-muted-foreground">Permission denied</p>
                    </div>
                </div>

                {dockerActions.length > 0 && (
                    <div className="space-y-3">
                        <div className="flex items-center justify-between">
                            <p className="text-sm font-medium">
                                Docker cleanup actions
                            </p>

                            <div className="flex gap-2">
                                <Badge variant="outline">
                                    {enabledActions.length} enabled
                                </Badge>
                                <Badge variant="secondary">
                                    {disabledActions.length} disabled
                                </Badge>
                            </div>
                        </div>

                        <div className="space-y-2">
                            {dockerActions.map((action) => (
                                <div
                                    key={action.command}
                                    className="rounded-md border bg-muted/20 p-3"
                                >
                                    <div className="flex items-center justify-between gap-3">
                                        <p className="font-medium">
                                            {action.name}
                                        </p>

                                        <Badge
                                            variant={
                                                action.enabled
                                                    ? "default"
                                                    : "secondary"
                                            }
                                        >
                                            {action.status}
                                        </Badge>
                                    </div>

                                    <p className="mt-1 text-sm text-muted-foreground">
                                        {action.reason}
                                    </p>

                                    <code className="mt-2 block rounded-md bg-muted px-2 py-1 text-xs">
                                        {action.command}
                                    </code>
                                </div>
                            ))}
                        </div>
                    </div>
                )}
            </CardContent>
        </Card>
    );
}

function StatTable({
                       title,
                       description,
                       items,
                   }: {
    title: string;
    description: string;
    items: CleanerStatItem[];
}) {
    if (!items || items.length === 0) {
        return null;
    }

    return (
        <Card>
            <CardHeader>
                <CardTitle>{title}</CardTitle>
                <CardDescription>{description}</CardDescription>
            </CardHeader>

            <CardContent>
                <div className="overflow-x-auto rounded-md border">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>Name</TableHead>
                                <TableHead>Files</TableHead>
                                <TableHead className="text-right">Size</TableHead>
                            </TableRow>
                        </TableHeader>

                        <TableBody>
                            {items.map((item) => (
                                <TableRow key={item.name}>
                                    <TableCell className="max-w-[420px] truncate font-mono text-xs">
                                        {item.name}
                                    </TableCell>

                                    <TableCell>
                                        {item.files_touched.toLocaleString()}
                                    </TableCell>

                                    <TableCell className="text-right font-medium">
                                        {formatBytes(item.bytes)}
                                    </TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                </div>
            </CardContent>
        </Card>
    );
}

export default function CleanerRunOverview({
                                               element,
                                               store = {},
                                           }: CleanerRunOverviewProps) {
    const context: { store: any; event?: any } = { store };

    const result: CleanerRunResult | undefined = element.from
        ? resolveFromPath(element.from, context)
        : undefined;

    if (!hasRunResult(result)) {
        return (
            <Card id={element.id} className="w-full">
                <CardContent className="flex flex-col items-center justify-center gap-3 p-8 text-center">
                    <div className="flex h-12 w-12 items-center justify-center rounded-full bg-muted">
                        <ShieldCheck className="h-6 w-6 text-muted-foreground" />
                    </div>

                    <div>
                        <h3 className="font-semibold">
                            No cleaner analysis yet
                        </h3>

                        <p className="text-sm text-muted-foreground">
                            Run the cleaner analysis to display reclaimable space,
                            modules, warnings and Docker cleanup actions.
                        </p>
                    </div>
                </CardContent>
            </Card>
        );
    }

    const summary = result!.summary!;
    const modules = result!.modules ?? [];
    const scope = result!.selected_scope ?? {};
    const metadata = result!.metadata ?? {};

    const cacheModule = modules.find((module) => module.id === "cache");
    const allWarnings = getAllWarnings(modules);

    return (
        <div id={element.id} className="w-full space-y-6">
            <Card>
                <CardHeader>
                    <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
                        <div>
                            <CardTitle className="text-2xl">
                                {element.title ?? "Griffon Cleaner"}
                            </CardTitle>

                            <CardDescription>
                                {scope.profile ?? "unknown"} profile ·{" "}
                                {summary.dry_run ? "Dry-run" : "Cleanup mode"} ·{" "}
                                {(scope.selected_file_types?.length ?? 0) === 0
                                    ? "All file types"
                                    : `${scope.selected_file_types?.length} file type(s)`} ·{" "}
                                {formatDate(metadata.generated_at)}
                            </CardDescription>
                        </div>

                        <div className="flex flex-wrap gap-2">
                            <Badge variant="outline">
                                {metadata.plugin_name ?? "griffon_cleaner"}
                            </Badge>

                            {metadata.plugin_version && (
                                <Badge variant="secondary">
                                    v{metadata.plugin_version}
                                </Badge>
                            )}

                            {summary.dry_run && (
                                <Badge variant="secondary">Dry run</Badge>
                            )}
                        </div>
                    </div>
                </CardHeader>

                <CardContent>
                    {summary.dry_run && (
                        <Alert>
                            <Info className="h-4 w-4" />
                            <AlertTitle>Dry-run mode enabled</AlertTitle>
                            <AlertDescription>
                                This analysis only estimates reclaimable space. No
                                file was deleted during this run.
                            </AlertDescription>
                        </Alert>
                    )}
                </CardContent>
            </Card>

            <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
                <SummaryCard
                    title="Reclaimable space"
                    value={formatBytes(summary.total_reclaimable_bytes)}
                    description="Estimated disk space that can be cleaned"
                    icon={<Database className="h-4 w-4" />}
                />

                <SummaryCard
                    title="Files analyzed"
                    value={summary.total_files_touched.toLocaleString()}
                    description="Files or entries touched by the scan"
                    icon={<HardDrive className="h-4 w-4" />}
                />

                <SummaryCard
                    title="Warnings"
                    value={String(summary.total_warnings)}
                    description={`${summary.total_permission_denied} permission denied`}
                    icon={<FileWarning className="h-4 w-4" />}
                />

                <SummaryCard
                    title="Duration"
                    value={`${summary.duration_ms} ms`}
                    description="Last cleaner execution time"
                    icon={<Clock className="h-4 w-4" />}
                />
            </div>

            <div className="grid gap-4 xl:grid-cols-2">
                {modules.map((module) => (
                    <ModuleCard key={module.id} module={module} />
                ))}
            </div>

            {cacheModule && (
                <div className="grid gap-4 xl:grid-cols-2">
                    <StatTable
                        title="Biggest paths"
                        description="Largest root paths detected during the cache analysis."
                        items={cacheModule.top_root_paths ?? []}
                    />

                    <StatTable
                        title="Biggest file types"
                        description="File extensions or groups using the most space."
                        items={cacheModule.top_file_types ?? []}
                    />
                </div>
            )}

            {allWarnings.length > 0 && (
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            <AlertTriangle className="h-5 w-5 text-muted-foreground" />
                            Warnings
                        </CardTitle>

                        <CardDescription>
                            Main warnings reported by the cleaner modules.
                        </CardDescription>
                    </CardHeader>

                    <CardContent className="space-y-2">
                        {allWarnings.map((warning, index) => (
                            <div
                                key={`${warning}-${index}`}
                                className="rounded-md border bg-muted/20 p-3 text-sm"
                            >
                                {warning}
                            </div>
                        ))}
                    </CardContent>
                </Card>
            )}

            {summary.total_errors === 0 && (
                <Alert>
                    <CheckCircle2 className="h-4 w-4" />
                    <AlertTitle>Analysis completed</AlertTitle>
                    <AlertDescription>
                        The cleaner finished without critical errors.
                    </AlertDescription>
                </Alert>
            )}

            {!summary.dry_run && summary.total_reclaimable_bytes > 0 && (
                <Alert>
                    <Trash2 className="h-4 w-4" />
                    <AlertTitle>Cleanup mode</AlertTitle>
                    <AlertDescription>
                        This run may have deleted files depending on the selected
                        cleaner profile.
                    </AlertDescription>
                </Alert>
            )}
        </div>
    );
}