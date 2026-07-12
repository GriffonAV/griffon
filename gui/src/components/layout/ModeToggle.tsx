import { Moon, Sun } from "lucide-react";

import { Button } from "@/components/ui/button";
import { useTheme } from "@/providers/ThemeProvider";
import { ToggleGroup, ToggleGroupItem } from "../ui/toggle-group";
import ThemesList from "./ThemesList";
import { useState } from "react";

export function ModeToggle() {
    const { setTheme } = useTheme();

    // return (
    //     <DropdownMenu>
    //         <DropdownMenuTrigger asChild>
    //             <Button variant="outline" className="cursor-pointer" size="icon">
    //                 <Sun className="h-[1.2rem] w-[1.2rem] scale-100 rotate-0 transition-all dark:scale-0 dark:-rotate-90" />
    //                 <Moon className="absolute h-[1.2rem] w-[1.2rem] scale-0 rotate-90 transition-all dark:scale-100 dark:rotate-0" />
    //                 <span className="sr-only">Toggle theme</span>
    //             </Button>
    //         </DropdownMenuTrigger>
    //         <DropdownMenuContent align="end">
    //             <DropdownMenuItem className="cursor-pointer" onClick={() => setTheme("light")}>Light</DropdownMenuItem>
    //             <DropdownMenuItem className="cursor-pointer" onClick={() => setTheme("dark")}>Dark</DropdownMenuItem>
    //         </DropdownMenuContent>
    //     </DropdownMenu>
    // );

    // simple button switching between sun and moon icons, no dropdown
    return (
        <Button variant="outline" className="cursor-pointer" size="icon" title="Switch Dark/Light" onClick={() => setTheme(document.documentElement.classList.contains("dark") ? "light" : "dark")}>
            <Sun className="h-[1.2rem] w-[1.2rem] scale-100 rotate-0 transition-all dark:scale-0 dark:-rotate-90" />
            <Moon className="absolute h-[1.2rem] w-[1.2rem] scale-0 rotate-90 transition-all dark:scale-100 dark:rotate-0" />
        </Button>
    );

}

export function ModeToggleGroup() {
    const { setTheme } = useTheme();

    return (
        <ToggleGroup type="single" className="cursor-pointer" defaultValue="top" variant="outline">
            <ToggleGroupItem
                value="top"
                aria-label="Dark"
                className="cursor-pointer"
                onClick={() => setTheme("dark")}
            >
                Dark <Moon className="ml-2" />
            </ToggleGroupItem>
            <ToggleGroupItem
                value="bottom"
                aria-label="Light"
                className="cursor-pointer"
                onClick={() => setTheme("light")}
            >
                Light <Sun className="ml-2" />
            </ToggleGroupItem>
        </ToggleGroup>
    );
}

export function ThemeInitializer() {
    function setTheme(themeName: string) {
        const existingTheme = document.getElementById("dynamic-theme");
        if (existingTheme) {
            existingTheme.remove();
        }

        const link = document.createElement("link");
        link.id = "dynamic-theme";
        link.rel = "stylesheet";
        link.href = `/themes/${themeName}.css`;
        document.head.appendChild(link);
    }

    useState(() => {
        const savedTheme = localStorage.getItem("theme") || "default";
        setTheme(savedTheme);
    });

    return null;
}

export function ChangeThemeButtonTest() {
    const [selectedTheme, setSelectedTheme] = useState<string>(() => {
        return localStorage.getItem("theme") || "default";
    });

    function switchTheme(themeName: string) {
        const existingTheme = document.getElementById("dynamic-theme");
        if (existingTheme) {
            existingTheme.remove();
        }

        const link = document.createElement("link");
        link.id = "dynamic-theme";
        link.rel = "stylesheet";
        link.href = `/themes/${themeName}.css`;
        document.head.appendChild(link);

        localStorage.setItem("theme", themeName);
        setSelectedTheme(themeName);
    }

    return (
        <div className="flex flex-col gap-3">
            <span>Color theme:</span>
            <div className="flex flex-wrap gap-2">
                {(Object.keys(ThemesList) as Array<keyof typeof ThemesList>).map((theme) => (
                    <div
                        key={theme}
                        className={`flex flex-row gap-1 items-center cursor-pointer px-7 py-1 hover:bg-muted rounded w-44 ${selectedTheme === theme ? "border-2 border-primary" : ""
                            }`}
                        style={themePreviewStyle(ThemesList[theme])}
                        onClick={() => switchTheme(theme)}
                    >
                        <div className="border size-3 bg-primary rounded-sm" />
                        <div className="border size-3 bg-secondary rounded-sm" />
                        <div className="border size-3 bg-accent rounded-sm" />
                        <span>{theme}</span>
                    </div>
                ))}
            </div>
        </div>
    );
}

function themePreviewStyle(theme: Record<string, string>): React.CSSProperties {
    return {
        "--primary": theme["--primary"],
        "--secondary": theme["--secondary"],
        "--accent": theme["--accent"],
    } as React.CSSProperties;
}
