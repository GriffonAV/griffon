import type { TabNavigationElement } from "@/components/types";
import { usePageTabs } from "@/providers/PageTabsContext";
import { ArrowRight } from "lucide-react";

type Props = {
  element: TabNavigationElement;
};

export default function GriffonTabNavigation({ element }: Props) {
  const pageTabs = usePageTabs();

  return (
    <div id={element.id} className="grid gap-3 md:grid-cols-2">
      {element.items.map((item) => {
        const isActive = pageTabs?.activeTab === item.target;
        const isAvailable = pageTabs?.tabs.includes(item.target) ?? false;

        return (
          <button
            key={`${element.id}-${item.target}`}
            type="button"
            disabled={!isAvailable || isActive}
            onClick={() => pageTabs?.selectTab(item.target)}
            className="group flex min-h-28 w-full items-center justify-between gap-4 rounded-xl border bg-card p-5 text-left shadow-sm transition-all hover:-translate-y-0.5 hover:border-primary/50 hover:shadow-md disabled:pointer-events-none disabled:opacity-60"
          >
            <span className="min-w-0">
              <span className="block text-base font-semibold">{item.label}</span>
              {item.description ? (
                <span className="mt-1 block text-sm leading-relaxed text-muted-foreground">
                  {item.description}
                </span>
              ) : null}
            </span>
            <ArrowRight className="size-5 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-1 group-hover:text-primary" />
          </button>
        );
      })}
    </div>
  );
}
