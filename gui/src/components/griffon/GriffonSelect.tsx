import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { GriffonActionHandler, SelectElement } from "@/components/types";
import { resolveFromPath } from "@/lib/utils";

type Props = {
  element: SelectElement;
  store?: Record<string, any>;
  onAction?: GriffonActionHandler;
};

export default function GriffonSelect({ element, store = {}, onAction }: Props) {
  const value = element.value_from
    ? String(resolveFromPath(element.value_from, { store }) ?? element.value ?? "")
    : element.value;

  return (
    <div id={element.id} className="flex flex-col gap-2">
      {element.label ? (
        <label htmlFor={element.id} className="text-sm font-medium leading-none">
          {element.label}
        </label>
      ) : null}

      <Select
        value={value}
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
        <SelectTrigger id={element.id} className="w-full">
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
