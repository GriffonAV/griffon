import { useEffect, useMemo, useRef, useState } from "react";
import { ChevronDown, Search, X } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { resolveFromPath } from "@/lib/utils";
import type { GriffonActionHandler } from "../types";

interface CleanerFileTypeSelectorProps {
    element: {
        type: "cleaner_file_type_selector";
        id: string;
        title?: string;
        description?: string;
        from?: string;
        selectedFrom?: string;
        action?: string;
    };
    store?: Record<string, any>;
    onAction?: GriffonActionHandler;
}

type FileTypeOption = {
    id: string;
    label: string;
};

type FileTypeOptionsData = {
    items: FileTypeOption[];
};

type SelectedFileTypesData = {
    file_types: string[];
};

function getSelectedFileTypes(data?: SelectedFileTypesData): string[] {
    return data?.file_types ?? [];
}

function emitSelection(
    nextSelected: string[],
    element: CleanerFileTypeSelectorProps["element"],
    onAction?: GriffonActionHandler
) {
    if (!element.action || !onAction) {
        return;
    }

    onAction(element.action, {
        ...element,
        value: {
            file_types: nextSelected,
        },
    });
}

export default function CleanerFileTypeSelector({
                                                    element,
                                                    store = {},
                                                    onAction,
                                                }: CleanerFileTypeSelectorProps) {
    const [open, setOpen] = useState(false);
    const [search, setSearch] = useState("");
    const rootRef = useRef<HTMLDivElement | null>(null);

    const context: { store: any; event?: any } = { store };

    const optionsData: FileTypeOptionsData = element.from
        ? resolveFromPath(element.from, context)
        : { items: [] };

    const selectedData: SelectedFileTypesData = element.selectedFrom
        ? resolveFromPath(element.selectedFrom, context)
        : { file_types: [] };

    const options = optionsData?.items ?? [];
    const selectedFileTypes = getSelectedFileTypes(selectedData);

    const filteredOptions = useMemo(() => {
        const normalizedSearch = search.trim().toLowerCase();

        if (!normalizedSearch) {
            return options;
        }

        return options.filter((option) => {
            return (
                option.id.toLowerCase().includes(normalizedSearch) ||
                option.label.toLowerCase().includes(normalizedSearch)
            );
        });
    }, [options, search]);

    const selectedLabels = useMemo(() => {
        return selectedFileTypes.map((selectedId) => {
            const option = options.find((item) => item.id === selectedId);
            return option?.label ?? selectedId;
        });
    }, [options, selectedFileTypes]);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (!rootRef.current) {
                return;
            }

            if (!rootRef.current.contains(event.target as Node)) {
                setOpen(false);
            }
        };

        document.addEventListener("mousedown", handleClickOutside);

        return () => {
            document.removeEventListener("mousedown", handleClickOutside);
        };
    }, []);

    const toggleFileType = (id: string) => {
        const exists = selectedFileTypes.includes(id);

        const nextSelected = exists
            ? selectedFileTypes.filter((item) => item !== id)
            : [...selectedFileTypes, id];

        emitSelection(nextSelected, element, onAction);
    };

    const selectAllVisible = () => {
        const visibleIds = filteredOptions.map((option) => option.id);

        const nextSelected = Array.from(
            new Set([...selectedFileTypes, ...visibleIds])
        );

        emitSelection(nextSelected, element, onAction);
    };

    const clearSelection = () => {
        emitSelection([], element, onAction);
    };

    const removeSelected = (id: string) => {
        emitSelection(
            selectedFileTypes.filter((item) => item !== id),
            element,
            onAction
        );
    };

    return (
        <div id={element.id} ref={rootRef} className="relative w-full">
            <div className="flex flex-col gap-2">
                <div className="flex flex-col gap-1">
                    <p className="text-sm font-medium">
                        {element.title ?? "Targeted file types"}
                    </p>

                    {element.description && (
                        <p className="text-xs text-muted-foreground">
                            {element.description}
                        </p>
                    )}
                </div>

                <Button
                    type="button"
                    variant="outline"
                    className="w-full justify-between"
                    onClick={() => setOpen((value) => !value)}
                >
                    <span className="truncate">
                        {selectedFileTypes.length === 0
                            ? "All file types"
                            : `${selectedFileTypes.length} file type(s) selected`}
                    </span>

                    <ChevronDown className="h-4 w-4 opacity-70" />
                </Button>

                {selectedFileTypes.length > 0 && (
                    <div className="flex flex-wrap gap-2">
                        {selectedLabels.slice(0, 4).map((label, index) => {
                            const id = selectedFileTypes[index];

                            return (
                                <Badge
                                    key={id}
                                    variant="secondary"
                                    className="gap-1"
                                >
                                    {label}

                                    <button
                                        type="button"
                                        onClick={() => removeSelected(id)}
                                        className="rounded-sm hover:bg-background/60"
                                    >
                                        <X className="h-3 w-3" />
                                    </button>
                                </Badge>
                            );
                        })}

                        {selectedLabels.length > 4 && (
                            <Badge variant="outline">
                                +{selectedLabels.length - 4}
                            </Badge>
                        )}
                    </div>
                )}
            </div>

            {open && (
                <div className="absolute z-50 mt-2 w-full rounded-md border bg-background shadow-lg">
                    <div className="space-y-3 p-3">
                        <div className="flex items-center gap-2 rounded-md border px-3 py-2">
                            <Search className="h-4 w-4 text-muted-foreground" />

                            <input
                                value={search}
                                onChange={(event) =>
                                    setSearch(event.target.value)
                                }
                                placeholder="Search file type..."
                                className="w-full bg-transparent text-sm outline-none placeholder:text-muted-foreground"
                            />
                        </div>

                        <div className="flex items-center justify-between gap-2">
                            <p className="text-xs text-muted-foreground">
                                {selectedFileTypes.length === 0
                                    ? "No filter: all file types are included"
                                    : `${selectedFileTypes.length} selected`}
                            </p>

                            <div className="flex gap-2">
                                <Button
                                    type="button"
                                    variant="outline"
                                    size="sm"
                                    disabled={filteredOptions.length === 0}
                                    onClick={selectAllVisible}
                                >
                                    Select visible
                                </Button>

                                <Button
                                    type="button"
                                    variant="outline"
                                    size="sm"
                                    disabled={selectedFileTypes.length === 0}
                                    onClick={clearSelection}
                                >
                                    Clear
                                </Button>
                            </div>
                        </div>

                        <div className="max-h-[280px] overflow-y-auto rounded-md border">
                            {filteredOptions.length === 0 ? (
                                <div className="p-4 text-center text-sm text-muted-foreground">
                                    No file type found.
                                </div>
                            ) : (
                                filteredOptions.map((option) => {
                                    const checked = selectedFileTypes.includes(
                                        option.id
                                    );

                                    return (
                                        <button
                                            key={option.id}
                                            type="button"
                                            onClick={() =>
                                                toggleFileType(option.id)
                                            }
                                            className={`flex w-full items-center justify-between gap-3 border-b px-3 py-2 text-left text-sm last:border-b-0 hover:bg-muted ${
                                                checked ? "bg-muted/70" : ""
                                            }`}
                                        >
                                            <div className="flex items-center gap-3">
                                                <Checkbox checked={checked} />

                                                <div>
                                                    <p className="font-medium">
                                                        {option.label}
                                                    </p>

                                                    <p className="text-xs text-muted-foreground">
                                                        {option.id}
                                                    </p>
                                                </div>
                                            </div>
                                        </button>
                                    );
                                })
                            )}
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}