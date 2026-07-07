import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { open } from "@tauri-apps/plugin-dialog";

interface GriffonFileSelectProps {
  element: {
    id: string;
    name?: string;
    accept_directory?: boolean;
    action?: string;
    [key: string]: any;
  };
  store?: Record<string, any>;
  onAction?: (action: string, event?: any) => void;
}

export default function GriffonFileSelect({
  element,
  onAction,
}: GriffonFileSelectProps) {

  async function handleClick() {
    const selected = await open({
      multiple: true,
      directory: element?.accept_directory || false,
      filters: element.accept
        ? [
            {
              name: "Files",
              extensions: element.accept
                .split(",")
                .map((e: string) => e.trim().replace(/^\./, "")),
            },
          ]
        : undefined,
    });

    if (!selected) return;

    const paths = Array.isArray(selected) ? selected : [selected];

    paths.forEach((path) => {
      console.log("path : ", path)
      if (element.action) {
        onAction?.(element.action, {
          value: path,
          append: true,
        });
      }
    });
  }

  return (
    <div className={cn("flex flex-col gap-2", element.className)}>
      <Button
        type="button"
        variant="outline"
        disabled={element.disabled}
        onClick={handleClick}
      >
        {element.name ?? "Select file"}
      </Button>
    </div>
  );
}