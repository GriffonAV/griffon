import React from "react";
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { Button } from "../ui/button";
import { Search } from "lucide-react";
import clsx from "clsx";
import { Kbd, KbdGroup } from "../ui/kbd";

function SearchInput() {
  const [open, setOpen] = React.useState(false);

  React.useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === "F" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setOpen((open) => !open);
      }
    };
    document.addEventListener("keydown", down);
    return () => document.removeEventListener("keydown", down);
  }, []);

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
            <CommandItem>test1</CommandItem>
            <CommandItem>test2</CommandItem>
            <CommandItem>test3</CommandItem>
          </CommandGroup>
        </CommandList>
      </CommandDialog>
    </div >
  );
}

export { SearchInput };
