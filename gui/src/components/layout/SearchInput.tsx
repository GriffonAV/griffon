import React from "react";
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { Link, useNavigate } from "react-router-dom";

import { Button } from "../ui/button";
import { Search } from "lucide-react";
import clsx from "clsx";
import { Kbd, KbdGroup } from "../ui/kbd";

function SearchInput() {
  const navigate = useNavigate();
  const [open, setOpen] = React.useState(false);

  // open with ctrl+f
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

  // if enter is pressed, close the dialog and navigate to the first command if it exists
  const handleCommandSelect = (command: string) => {
    setOpen(false);
    // navigate to the command with react-router-dom
    navigate(`/${command.toLowerCase()}`);

  }


  return (
    <div>
      <Button
        variant="ghost"
        className={clsx(
          "w-full justify-start cursor-pointer mb-2 font-bold"

        )}
        onClick={() => setOpen((open) => !open)}
      >
        <span className="mr-2">{<Search />}</span>
        <KbdGroup>
          <Kbd>Ctrl</Kbd>
          <span>+</span>
          <Kbd>F</Kbd>
        </KbdGroup>
      </Button>
      <CommandDialog open={open} onOpenChange={setOpen}>
        <CommandInput placeholder="Search for any plugin or command..." />
        <CommandList>
          <CommandEmpty>No results found.</CommandEmpty>
          <CommandGroup heading="Suggestions">
            <CommandItem onSelect={() => handleCommandSelect("dashboard")}>
              Dashboard
            </CommandItem>
            <CommandItem onSelect={() => handleCommandSelect("log")}>
              Logs
            </CommandItem>
          </CommandGroup>
          <CommandGroup heading="Plugins">
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
