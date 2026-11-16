import { createContext, useContext } from "react";

type PageTabsContextValue = {
  activeTab?: string;
  tabs: readonly string[];
  selectTab: (tab: string) => boolean;
};

const PageTabsContext = createContext<PageTabsContextValue | null>(null);

export const PageTabsProvider = PageTabsContext.Provider;

export function usePageTabs() {
  return useContext(PageTabsContext);
}
