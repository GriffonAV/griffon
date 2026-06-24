import { createContext, useContext, useEffect, useState, type ReactNode } from "react";

type SidebarContextValue = {
    isCollapsed: boolean;
    setIsCollapsed: (value: boolean) => void;
    toggleSidebar: () => void;
};

const SidebarContext = createContext<SidebarContextValue | undefined>(undefined);

export function SidebarProvider({ children }: { children: ReactNode }) {
    const [isCollapsed, setIsCollapsed] = useState(false);

    // collapse sidebar when window width is less than 768px
    // and expand it when window width is greater than 768px
    useState(() => {
        const handleResize = () => {
            if (window.innerWidth < 768) {
                setIsCollapsed(true);
            } else {
                setIsCollapsed(false);
            }
        };

        window.addEventListener("resize", handleResize);

        // call handleResize on mount to set the initial state
        handleResize();

        return () => {
            window.removeEventListener("resize", handleResize);
        };
    });

    // collapse sidebar with ctrl + b
    useEffect(() => {
        const handleKeyDown = (event: KeyboardEvent) => {
            if (event.ctrlKey && event.key === "b") {
                setIsCollapsed((prev) => !prev);
            }
        };

        window.addEventListener("keydown", handleKeyDown);
        return () => {
            window.removeEventListener("keydown", handleKeyDown);
        };
    }, []);


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