import { createContext, useContext, useState, type ReactNode } from "react";

type SidebarContextValue = {
    isCollapsed: boolean;
    setIsCollapsed: (value: boolean) => void;
    toggleSidebar: () => void;
};

const SidebarContext = createContext<SidebarContextValue | undefined>(undefined);

export function SidebarProvider({ children }: { children: ReactNode }) {
    const [isCollapsed, setIsCollapsed] = useState(false);

    return (
        <SidebarContext.Provider
            value={{
                isCollapsed,
                setIsCollapsed,
                toggleSidebar: () => setIsCollapsed((prev) => !prev),
            }}
        >
            {children}
        </SidebarContext.Provider>
    );
}

export function useSidebar() {
    const context = useContext(SidebarContext);

    if (!context) {
        throw new Error("useSidebar must be used within a SidebarProvider");
    }

    return context;
}