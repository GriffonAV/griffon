import React from "react";
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { useNavigate } from "react-router-dom";

import { Button } from "../ui/button";
import { Search } from "lucide-react";
import clsx from "clsx";
import { Kbd, KbdGroup } from "../ui/kbd";

import { usePlugins } from "@/bindings/PluginContext";

function SearchInput({ isCollapsed }: { isCollapsed: boolean }) {
  const navigate = useNavigate();
  const [open, setOpen] = React.useState(false);

  const { plugins } = usePlugins();

  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === "f") {
        e.preventDefault();
        setOpen((open) => !open);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  const handleCommandSelect = (command: string) => {
    setOpen(false);
    navigate(`/${command.toLowerCase()}`);
  }

  const handlePluginSelect = (fileName: string) => {
    setOpen(false);
    navigate(`/plugin/${fileName}`);
  }

  return (
    <div>
      <Button
        title="Search (Ctrl + F)"
        variant="ghost"
        className={clsx(
          "w-full cursor-pointer font-bold overflow-hidden",
          isCollapsed ? "justify-center px-0" : "justify-start"
        )}
        onClick={() => setOpen((open) => !open)}
      >
        <span className={clsx(isCollapsed ? "mr-0" : "mr-2")}>
          <Search />
        </span>

        {!isCollapsed && (
          <KbdGroup>
            <Kbd>Ctrl</Kbd>
            <span>+</span>
            <Kbd>F</Kbd>
          </KbdGroup>
        )}
      </Button>
      <CommandDialog open={open} onOpenChange={setOpen}>
        <CommandInput placeholder="Search for any plugin or command..." />
        <CommandList>
          <CommandEmpty>No results found.</CommandEmpty>
          <CommandGroup heading="Suggestions">
            <CommandItem onSelect={() => handleCommandSelect("dashboard")}>
              Overview
            </CommandItem>
            <CommandItem onSelect={() => handleCommandSelect("log")}>
              Activity Log
            </CommandItem>
          </CommandGroup>

          <CommandGroup heading="Extensions">
            {plugins.map((plugin) => {
              const isAvailable = plugin.status === "healthy";

              return (
                <CommandItem
                  key={plugin.uuid}
                  disabled={!isAvailable}
                  onSelect={() => {
                    if (isAvailable) {
                      handlePluginSelect(plugin.file_name);
                    }
                  }}
                >
                  <span>{plugin.display_name}</span>
                  {!isAvailable && (
                    <span className="ml-auto text-xs text-muted-foreground">
                      {plugin.status === "busy"
                        ? "Updating"
                        : plugin.status === "disabled"
                          ? "Disabled"
                          : "Unavailable"}
                    </span>
                  )}
                </CommandItem>
              );
            })}
          </CommandGroup>

          <CommandGroup heading="Settings">
            <CommandItem onSelect={() => handleCommandSelect("settings")}>
              Settings
            </CommandItem>
          </CommandGroup>

        </CommandList>
      </CommandDialog>
    </div >
  );
}

export { SearchInput };
