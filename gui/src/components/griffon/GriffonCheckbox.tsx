import { Checkbox } from "@/components/ui/checkbox";
import type { GriffonActionHandler, CheckboxElement } from "@/components/types";

type Props = {
  element: CheckboxElement;
  onAction?: GriffonActionHandler;
};

export default function GriffonCheckbox({ element, onAction }: Props) {
  return (
    <div id={element.id} className="flex items-start gap-3">
      <Checkbox
        id={`${element.id}-checkbox`}
        defaultChecked={!!element.checked}
        disabled={element.disabled}
        onCheckedChange={(checked) => {
          if (element.action && onAction) {
            onAction(element.action, {
              ...element,
              checked: checked === true,
            });
          }
        }}
      />

      <div className="grid gap-1.5 leading-none">
        <label
          htmlFor={`${element.id}-checkbox`}
          className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
        >
          {element.label}
        </label>

        {element.description ? (
          <p className="text-xs text-muted-foreground">{element.description}</p>
        ) : null}
      </div>
    </div>
  );
}