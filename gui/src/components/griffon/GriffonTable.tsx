import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type {
  TableColumn,
  TableElement,
  TableRowData,
  Tone,
} from "@/components/types";
import { cn, resolveFromPath } from "@/lib/utils";
import { Checkbox } from "../ui/checkbox";
import { Input } from "../ui/input";
import { useMemo, useState } from "react";

type Props = {
  element: TableElement;
  store: Record<string, any>;
  onAction?: (action: string, event?: any) => void;
};

function getTransmitValues(element: TableElement, row: TableRowData) {
  return element.columns
    ?.filter((column) => column.transmit)
    .map((column) => row[column.key]);
}

const TONE_STYLES: Record<Tone, { badge: string; border: string }> = {
  neutral: {
    badge: "border-border bg-muted text-muted-foreground",
    border: "border-l-border",
  },
  info: {
    badge: "border-blue-500/30 bg-blue-500/10 text-blue-700 dark:text-blue-300",
    border: "border-l-blue-500",
  },
  success: {
    badge:
      "border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300",
    border: "border-l-emerald-500",
  },
  warning: {
    badge:
      "border-amber-500/30 bg-amber-500/10 text-amber-800 dark:text-amber-300",
    border: "border-l-amber-500",
  },
  danger: {
    badge: "border-red-500/30 bg-red-500/10 text-red-700 dark:text-red-300",
    border: "border-l-red-500",
  },
};

function getTone(column: TableColumn, value: string): Tone {
  return column.tone_map?.[value] ?? column.tone ?? "neutral";
}

function CellValue({ column, value }: { column: TableColumn; value: string }) {
  if (column.cell === "badge") {
    return (
      <span
        className={cn(
          "inline-flex rounded-full border px-2.5 py-1 text-xs font-semibold",
          TONE_STYLES[getTone(column, value)].badge,
        )}
      >
        {value}
      </span>
    );
  }
  if (column.cell === "code") {
    return (
      <code className="break-all rounded-md bg-muted px-2 py-1 font-mono text-xs text-foreground">
        {value}
      </code>
    );
  }
  return (
    <span className={column.cell === "primary" ? "font-semibold" : undefined}>
      {value}
    </span>
  );
}

export default function GriffonTable({ element, store = {}, onAction }: Props) {
  const rows = useMemo<TableRowData[]>(() => {
    const storeData = element.from
      ? resolveFromPath(element.from, { store })
      : null;
    return Array.isArray(storeData) ? storeData : (element.rows ?? []);
  }, [element.from, element.rows, store]);
  const [query, setQuery] = useState("");
  const [filters, setFilters] = useState<Record<string, string>>({});
  const visibleColumns = useMemo(
    () => element.columns?.filter((column) => !column.hide) ?? [],
    [element.columns],
  );
  const cardColumns = useMemo(() => {
    const badgeColumns = visibleColumns.filter(
      (column) => column.cell === "badge",
    );
    const primaryColumn =
      visibleColumns.find((column) => column.cell === "primary") ??
      visibleColumns.find((column) => column.cell !== "badge");
    const detailColumns = visibleColumns.filter(
      (column) => column !== primaryColumn && column.cell !== "badge",
    );

    return { badgeColumns, primaryColumn, detailColumns };
  }, [visibleColumns]);

  const filterKeys = useMemo(
    () =>
      element.filter_keys ??
      (element.filter_key ? [element.filter_key] : []),
    [element.filter_key, element.filter_keys],
  );
  const filterOptions = useMemo(
    () =>
      Object.fromEntries(
        filterKeys.map((key) => [
          key,
          Array.from(
            new Set(rows.map((row) => String(row[key] ?? "")).filter(Boolean)),
          ).sort(),
        ]),
      ) as Record<string, string[]>,
    [filterKeys, rows],
  );
  const activeFilters = useMemo(
    () =>
      Object.fromEntries(
        filterKeys.map((key) => {
          const value = filters[key] ?? "";
          return [key, filterOptions[key]?.includes(value) ? value : ""];
        }),
      ) as Record<string, string>,
    [filterKeys, filterOptions, filters],
  );

  const visibleRows = useMemo(() => {
    const normalizedQuery = query.trim().toLocaleLowerCase();

    return rows.filter((row) => {
      if (
        filterKeys.some(
          (key) =>
            activeFilters[key] &&
            String(row[key] ?? "") !== activeFilters[key],
        )
      ) {
        return false;
      }
      if (!normalizedQuery) return true;
      return visibleColumns.some((column) =>
        String(row[column.key] ?? "")
          .toLocaleLowerCase()
          .includes(normalizedQuery),
      );
    });
  }, [activeFilters, filterKeys, query, rows, visibleColumns]);

  const columnCount =
    visibleColumns.length + (element.selectable !== false ? 1 : 0);

  return (
    <div id={element.id} className="w-full space-y-3">
      {element.searchable || filterKeys.length > 0 ? (
        <div className="rounded-lg border bg-muted/20 p-3">
          <div className="flex flex-col gap-2 lg:flex-row">
            {element.searchable ? (
              <Input
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="Search results…"
                aria-label="Search table results"
                className="bg-background lg:max-w-md"
              />
            ) : null}
            {filterKeys.map((key) => {
              const label =
                visibleColumns.find((column) => column.key === key)?.label ?? key;
              return (
                <select
                  key={key}
                  value={activeFilters[key] ?? ""}
                  onChange={(event) =>
                    setFilters((current) => ({
                      ...current,
                      [key]: event.target.value,
                    }))
                  }
                  aria-label={`Filter by ${label}`}
                  className="h-9 rounded-md border border-input bg-background px-3 text-sm shadow-xs"
                >
                  <option value="">All {label.toLocaleLowerCase()}</option>
                  {filterOptions[key]?.map((option) => (
                    <option key={option} value={option}>
                      {option}
                    </option>
                  ))}
                </select>
              );
            })}
            {query || Object.values(activeFilters).some(Boolean) ? (
              <button
                type="button"
                onClick={() => {
                  setQuery("");
                  setFilters({});
                }}
                className="h-9 rounded-md px-3 text-sm font-medium text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
              >
                Clear filters
              </button>
            ) : null}
          </div>
          {element.show_count !== false ? (
            <p className="mt-2 text-xs text-muted-foreground" aria-live="polite">
              {visibleRows.length} of {rows.length} result
              {rows.length === 1 ? "" : "s"}
            </p>
          ) : null}
        </div>
      ) : null}
      {element.layout === "cards" ? (
        <div className="space-y-3">
          {visibleRows.map((row, rowIndex) => {
            const { badgeColumns, primaryColumn, detailColumns } = cardColumns;
            const leadingBadge = badgeColumns[0];
            const leadingValue = leadingBadge
              ? String(row[leadingBadge.key] ?? "")
              : "";
            const tone = leadingBadge
              ? getTone(leadingBadge, leadingValue)
              : "neutral";

            return (
              <article
                key={row.id ?? rowIndex}
                className={cn(
                  "rounded-xl border border-l-4 bg-card p-4 shadow-sm",
                  TONE_STYLES[tone].border,
                )}
              >
                <div className="flex items-start gap-3">
                  {element.selectable !== false ? (
                    <Checkbox
                      className="mt-1"
                      onCheckedChange={(checked) => {
                        onAction?.(element.action ?? "", {
                          value: getTransmitValues(element, row),
                          checked,
                        });
                      }}
                    />
                  ) : null}
                  <div className="min-w-0 flex-1">
                    <div className="flex flex-wrap items-center gap-2">
                      {badgeColumns.map((column) => (
                        <CellValue
                          key={column.key}
                          column={column}
                          value={String(row[column.key] ?? "")}
                        />
                      ))}
                    </div>
                    {primaryColumn ? (
                      <h3 className="mt-3 text-base font-semibold leading-snug text-foreground">
                        {String(row[primaryColumn.key] ?? "")}
                      </h3>
                    ) : null}
                    {detailColumns.length > 0 ? (
                      <dl className="mt-4 grid gap-4 xl:grid-cols-2">
                        {detailColumns.map((column) => (
                          <div key={column.key} className="min-w-0">
                            <dt className="mb-1.5 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                              {column.label}
                            </dt>
                            <dd className="text-sm leading-relaxed text-foreground/90">
                              <CellValue
                                column={column}
                                value={String(row[column.key] ?? "")}
                              />
                            </dd>
                          </div>
                        ))}
                      </dl>
                    ) : null}
                  </div>
                </div>
              </article>
            );
          })}
          {visibleRows.length === 0 ? (
            <div className="rounded-xl border border-dashed px-6 py-12 text-center text-sm text-muted-foreground">
              {element.empty_message ?? "No results."}
            </div>
          ) : null}
        </div>
      ) : (
        <div className="w-full overflow-x-auto rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                {element.selectable !== false ? (
                  <TableHead key="select">Select</TableHead>
                ) : null}
                {visibleColumns.map((column) => (
                  <TableHead key={column.key}>{column.label}</TableHead>
                ))}
              </TableRow>
            </TableHeader>

            <TableBody>
              {visibleRows.map((row, rowIndex) => (
                <TableRow key={row.id ?? rowIndex}>
                  {element.selectable !== false ? (
                    <TableCell>
                      <Checkbox
                        onCheckedChange={(checked) => {
                          onAction?.(element.action ?? "", {
                            value: getTransmitValues(element, row),
                            checked,
                          });
                        }}
                      />
                    </TableCell>
                  ) : null}
                  {visibleColumns.map((column) => (
                    <TableCell key={`${rowIndex}-${column.key}`}>
                      <CellValue
                        column={column}
                        value={String(row[column.key] ?? "")}
                      />
                    </TableCell>
                  ))}
                </TableRow>
              ))}
              {visibleRows.length === 0 ? (
                <TableRow>
                  <TableCell
                    colSpan={Math.max(columnCount, 1)}
                    className="h-24 text-center text-muted-foreground"
                  >
                    {element.empty_message ?? "No results."}
                  </TableCell>
                </TableRow>
              ) : null}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  );
}
