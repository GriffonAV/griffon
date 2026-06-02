import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";

import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import { Folder, File } from "lucide-react";
import { resolveFromPath } from "@/lib/utils";
import type { GriffonActionHandler } from "../types";

interface CleanerCandidateListProps {
    element: {
        type: "cleaner_candidate_list";
        id: string;
        title?: string;
        from?: string;
        selectedFrom?: string;
        optionsFrom?: string;
        action?: string;
        deleteAction?: string;
        value?: {
            paths: string[];
            dry_run?: boolean;
            file_types?: string[];
        };
    };
    store?: Record<string, any>;
    onAction?: GriffonActionHandler;
}

type CleanerOptionsData = {
    dry_run?: boolean;
    file_types?: string[];
};

type CleanerCandidate = {
    path: string;
    name: string;
    category: string;
    kind: "file" | "directory";
    size: number;
    file_type?: string;
};

type CleanerCandidatesData = {
    ok: boolean;
    items: CleanerCandidate[];
};

type FilesSelectedData = {
    paths: string[];
    dry_run?: boolean;
    file_types?: string[];
};

function formatBytes(bytes: number): string {
    if (!bytes || bytes <= 0) {
        return "0 B";
    }

    const units = ["B", "KB", "MB", "GB", "TB"];
    const index = Math.floor(Math.log(bytes) / Math.log(1024));
    const value = bytes / Math.pow(1024, index);

    return `${value.toFixed(value >= 10 ? 1 : 2)} ${units[index]}`;
}

function isSelected(
    candidate: CleanerCandidate,
    selectedPaths: string[]
): boolean {
    return selectedPaths.includes(candidate.path);
}

function toggleCandidate(
    candidate: CleanerCandidate,
    element: CleanerCandidateListProps["element"],
    onAction?: GriffonActionHandler,
    selectedPaths: string[] = [],
    dryRun: boolean = true,
    fileTypes: string[] = []
) {
    if (!element?.action || !onAction) {
        return;
    }

    const exists = selectedPaths.includes(candidate.path);

    const nextSelected = exists
        ? selectedPaths.filter((path) => path !== candidate.path)
        : [...selectedPaths, candidate.path];

    onAction(element.action, {
        ...element,
        value: {
            paths: nextSelected,
            dry_run: dryRun,
            file_types: fileTypes,
        },
    });
}

function selectAll(
    candidates: CleanerCandidate[],
    element: CleanerCandidateListProps["element"],
    onAction?: GriffonActionHandler,
    dryRun: boolean = true,
    fileTypes: string[] = []
) {
    if (!element?.action || !onAction) {
        return;
    }

    onAction(element.action, {
        ...element,
        value: {
            paths: candidates.map((candidate) => candidate.path),
            dry_run: dryRun,
            file_types: fileTypes,
        },
    });
}

function clearSelection(
    element: CleanerCandidateListProps["element"],
    onAction?: GriffonActionHandler,
    dryRun: boolean = true,
    fileTypes: string[] = []
) {
    if (!element?.action || !onAction) {
        return;
    }

    onAction(element.action, {
        ...element,
        value: {
            paths: [],
            dry_run: dryRun,
            file_types: fileTypes,
        },
    });
}

function deleteCandidateNow(
    candidate: CleanerCandidate,
    element: CleanerCandidateListProps["element"],
    onAction?: GriffonActionHandler,
    dryRun: boolean = true,
    fileTypes: string[] = []
) {
    if (!element.deleteAction || !onAction) {
        return;
    }

    onAction(element.deleteAction, {
        ...element,
        value: {
            paths: [candidate.path],
            dry_run: dryRun,
            file_types: fileTypes,
        },
    });
}

export default function CleanerCandidateList({
                                                 element,
                                                 store = {},
                                                 onAction,
                                             }: CleanerCandidateListProps) {
    const context: { store: any; event?: any } = { store };

    const data: CleanerCandidatesData = element.from
        ? resolveFromPath(element.from, context)
        : { ok: false, items: [] };

    const selectedData: FilesSelectedData = element.selectedFrom
        ? resolveFromPath(element.selectedFrom, context)
        : { paths: [] };

    const optionsData: CleanerOptionsData = element.optionsFrom
        ? resolveFromPath(element.optionsFrom, context)
        : { dry_run: true, file_types: [] };

    const dryRun = optionsData?.dry_run ?? true;
    const fileTypes = optionsData?.file_types ?? [];

    const selectedPaths = selectedData?.paths ?? [];

    const candidates = data?.items ?? [];

    const selectedTotalSize = candidates
        .filter((candidate) => selectedPaths.includes(candidate.path))
        .reduce((total, item) => total + (item.size ?? 0), 0);

    return (
        <div id={element.id} className="w-full space-y-4">
            <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <div>
                    <h2 className="text-xl font-semibold">
                        {element.title ?? "Cleaner candidates"}
                    </h2>

                    <p className="text-sm text-muted-foreground">
                        Select the cache items you want to delete.
                    </p>
                </div>

                <div className="flex items-center gap-2">
                    <Button
                        variant="outline"
                        size="sm"
                        disabled={candidates.length === 0}
                        onClick={() =>
                            selectAll(candidates, element, onAction, dryRun, fileTypes)
                        }                    >
                        Select all
                    </Button>

                    <Button
                        variant="outline"
                        size="sm"
                        disabled={selectedPaths.length === 0}
                        onClick={() => clearSelection(element, onAction, dryRun, fileTypes)}
                    >
                        Clear
                    </Button>
                </div>
            </div>

            <div className="flex flex-col gap-2 rounded-md border bg-muted/30 p-3 md:flex-row md:items-center md:justify-between">
                <p className="text-sm">
                    Selected:{" "}
                    <span className="font-semibold">
                        {selectedPaths.length}
                    </span>{" "}
                    item(s)
                </p>

                <p className="text-sm text-muted-foreground">
                    Selected size:{" "}
                    <span className="font-semibold text-foreground">
                        {formatBytes(selectedTotalSize)}
                    </span>
                </p>
            </div>

            {candidates.length === 0 ? (
                <div className="rounded-md border border-dashed p-8 text-center text-sm text-muted-foreground">
                    No cleaner candidate loaded yet. Click the refresh button to
                    scan cache files.
                </div>
            ) : (
                <div className="w-full overflow-x-auto rounded-md border">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead className="w-[48px]"></TableHead>
                                <TableHead>Name</TableHead>
                                <TableHead>Category</TableHead>
                                <TableHead>Kind</TableHead>
                                <TableHead>Size</TableHead>
                                <TableHead>Path</TableHead>
                                <TableHead>File type</TableHead>
                                <TableHead>Kind</TableHead>
                                <TableHead>Action</TableHead>
                            </TableRow>
                        </TableHeader>

                        <TableBody>
                            {candidates.map((candidate, index) => {
                                const selected = isSelected(
                                    candidate,
                                    selectedPaths
                                );

                                return (
                                    <TableRow
                                        key={candidate.path ?? index}
                                        className={`cursor-pointer ${
                                            selected ? "bg-muted" : ""
                                        }`}
                                        onClick={() =>
                                            toggleCandidate(
                                                candidate,
                                                element,
                                                onAction,
                                                selectedPaths,
                                                dryRun
                                            )
                                        }
                                    >
                                        <TableCell
                                            onClick={(event) =>
                                                event.stopPropagation()
                                            }
                                        >
                                            <Checkbox
                                                checked={selected}
                                                onCheckedChange={() =>
                                                    toggleCandidate(
                                                        candidate,
                                                        element,
                                                        onAction,
                                                        selectedPaths,
                                                        dryRun
                                                    )
                                                }
                                            />
                                        </TableCell>

                                        <TableCell>
                                            <div className="flex items-center gap-2 font-medium">
                                                {candidate.kind ===
                                                "directory" ? (
                                                    <Folder className="h-4 w-4 text-muted-foreground" />
                                                ) : (
                                                    <File className="h-4 w-4 text-muted-foreground" />
                                                )}

                                                <span>{candidate.name}</span>
                                            </div>
                                        </TableCell>

                                        <TableCell>
                                            <Badge variant="secondary">
                                                {candidate.category}
                                            </Badge>
                                        </TableCell>

                                        <TableCell>
                                            <Badge variant="outline">
                                                {candidate.kind}
                                            </Badge>
                                        </TableCell>

                                        <TableCell className="font-medium">
                                            {formatBytes(candidate.size)}
                                        </TableCell>

                                        <TableCell className="max-w-[420px] truncate text-xs text-muted-foreground">
                                            {candidate.path}
                                        </TableCell>
                                        <TableCell>
                                            <Badge variant="outline">
                                                {candidate.file_type ?? "unknown"}
                                            </Badge>
                                        </TableCell>
                                        <TableCell>
                                            <Badge variant="outline">
                                                {candidate.kind}
                                            </Badge>
                                        </TableCell>
                                        <TableCell onClick={(event) => event.stopPropagation()}>
                                            <Button
                                                variant={dryRun ? "outline" : "destructive"}
                                                size="sm"
                                                onClick={() =>
                                                    deleteCandidateNow(
                                                        candidate,
                                                        element,
                                                        onAction,
                                                        dryRun,
                                                        fileTypes
                                                    )
                                                }
                                            >
                                                {dryRun ? "Test delete" : "Delete now"}
                                            </Button>
                                        </TableCell>
                                    </TableRow>
                                );
                            })}
                        </TableBody>
                    </Table>
                </div>
            )}

            <div className="rounded-md border bg-muted/20 p-3">
                <p className="mb-2 text-sm font-medium">Selected payload</p>

                <pre className="max-h-[220px] overflow-auto text-xs text-muted-foreground">
                    {JSON.stringify(selectedData ?? { files: [] }, null, 2)}
                </pre>
            </div>
        </div>
    );
}