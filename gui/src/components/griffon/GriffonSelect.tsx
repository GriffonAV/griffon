import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { GriffonActionHandler, SelectElement } from "@/components/types";

type Props = {
  element: SelectElement;
  onAction?: GriffonActionHandler;
};

export default function GriffonSelect({ element, onAction }: Props) {
  return (
    <div id={element.id} className="flex flex-col gap-2">
      {element.label ? (
        <label className="text-sm font-medium leading-none">
          {element.label}
        </label>
      ) : null}

      <Select
        defaultValue={element.value}
        disabled={element.disabled}
        onValueChange={(value) => {
          if (element.action && onAction) {
            onAction(element.action, {
              ...element,
              value,
            });
          }
        }}
      >
        <SelectTrigger className="w-full">
          <SelectValue placeholder={element.placeholder ?? "Select an option"} />
        </SelectTrigger>

        <SelectContent>
          {element.options?.map((option) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      {element.description ? (
        <p className="text-xs text-muted-foreground">{element.description}</p>
      ) : null}
    </div>
  );
}