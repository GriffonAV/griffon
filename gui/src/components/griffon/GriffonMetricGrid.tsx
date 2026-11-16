import type { MetricGridElement, Tone } from "@/components/types";
import { cn, resolveFromPath } from "@/lib/utils";

type Props = {
  element: MetricGridElement;
  store: Record<string, unknown>;
};

const TONE_CLASSES: Record<Tone, string> = {
  neutral: "border-border bg-card text-card-foreground",
  info: "border-blue-500/35 bg-blue-500/10 text-blue-700 dark:text-blue-300",
  success: "border-emerald-500/35 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300",
  warning: "border-amber-500/35 bg-amber-500/10 text-amber-700 dark:text-amber-300",
  danger: "border-red-500/35 bg-red-500/10 text-red-700 dark:text-red-300",
};

export default function GriffonMetricGrid({ element, store }: Props) {
  return (
    <div
      id={element.id}
      className="grid grid-cols-2 gap-3 md:grid-cols-3 xl:grid-cols-5"
    >
      {element.items.map((item) => {
        const value = resolveFromPath(item.from, { store });
        const numericValue = Number(value);
        const isZero = Number.isFinite(numericValue) && numericValue === 0;
        const tone = isZero ? (item.zero_tone ?? "neutral") : (item.tone ?? "neutral");

        return (
          <div
            key={`${element.id}-${item.label}`}
            className={cn(
              "min-w-0 rounded-xl border p-4 shadow-sm transition-colors",
              TONE_CLASSES[tone]
            )}
          >
            <div className="truncate text-xs font-semibold uppercase tracking-wide opacity-75">
              {item.label}
            </div>
            <div className="mt-2 text-3xl font-bold tabular-nums">
              {value ?? "—"}
            </div>
          </div>
        );
      })}
    </div>
  );
}
