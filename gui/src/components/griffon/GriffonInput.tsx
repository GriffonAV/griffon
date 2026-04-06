import { Input } from "@/components/ui/input";
import type { GriffonActionHandler, InputElement } from "@/components/types";

type Props = {
  element: InputElement;
  onAction?: GriffonActionHandler;
};

export default function GriffonInput({ element, onAction }: Props) {
  return (
    <div id={element.id} className="flex flex-col gap-2">
      {element.label ? (
        <label
          htmlFor={element.id}
          className="text-sm font-medium leading-none"
        >
          {element.label}
        </label>
      ) : null}

      <Input
        id={element.id}
        type={element.input_type ?? "text"}
        placeholder={element.placeholder}
        defaultValue={element.value}
        disabled={element.disabled}
        onChange={(e) => {
          if (element.action && onAction) {
            onAction(element.action, {
              ...element,
              value: e.target.value,
            });
          }
        }}
      />

      {element.description ? (
        <p className="text-xs text-muted-foreground">{element.description}</p>
      ) : null}
    </div>
  );
}