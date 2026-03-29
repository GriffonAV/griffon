import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

interface GriffonFileSelectProps {
  element: {
    id: string;
    name?: string;
    placeholder?: string;
    accept?: string;
    button_label?: string;
    disabled?: boolean;
    className?: string;
    action?: string;
    [key: string]: any;
  };
  store?: Record<string, any>;
  onAction?: (action: string, event?: any) => void;
}

export default function GriffonFileSelect({
  element,
  store = {},
  onAction,
}: GriffonFileSelectProps) {
  const selectedPath =
    typeof element.bind === "string" ? store[element.bind] : undefined;

  function handleChange(event: React.ChangeEvent<HTMLInputElement>) {
    const files = event.target.files;
    const file = files?.[0];
      
    if (!file) return;
    
    if (element.action) {
      onAction?.(element.action, {
        source: element.id,
        type: "file_select",
        file,
        files: Array.from(files),
        value: file.name,
        name: file.name,
      });
    }
  }

  return (
    <div className={cn("flex flex-col gap-2", element.className)}>
      {element.name ? (
        <label htmlFor={element.id} className="text-sm font-medium">
          {element.name}
        </label>
      ) : null}

      <div className="flex items-center gap-2">
        <Input
          value={selectedPath ?? ""}
          readOnly
          placeholder={element.placeholder ?? "No file selected"}
          disabled={element.disabled}
        />

        <Button
          type="button"
          variant="outline"
          disabled={element.disabled}
          onClick={() => {
            const input = document.getElementById(element.id) as HTMLInputElement | null;
            input?.click();
          }}
        >
          {element.button_label ?? "Browse"}
        </Button>
      </div>

      TEST

      <input
        id={element.id}
        type="file"
        className="hidden"
        accept={element.accept}
        disabled={element.disabled}
        onChange={handleChange}
      />
    </div>
  );
}