import { Switch } from "@/components/ui/switch";
import type { GriffonActionHandler, SwitchElement } from "@/components/types";
import { resolveFromPath } from "@/lib/utils";

type Props = {
  element: SwitchElement;
  store?: Record<string, any>;
  onAction?: GriffonActionHandler;
};

export default function GriffonSwitch({ element, store = {}, onAction }: Props) {
  const checked = element.checked_from
    ? Boolean(resolveFromPath(element.checked_from, { store }))
    : element.checked;

  return (
    <div
      id={element.id}
      className="flex items-center justify-between gap-4 rounded-lg border p-3"
    >
      <div className="flex flex-col gap-1">
        <label htmlFor={`${element.id}-control`} className="cursor-pointer text-sm font-medium leading-none">{element.label}</label>
        {element.description ? (
          <p className="text-xs text-muted-foreground">{element.description}</p>
        ) : null}
      </div>

      <Switch
        id={`${element.id}-control`}
        checked={checked}
        disabled={element.disabled}
        onCheckedChange={(checked) => {
          if (element.action && onAction) {
            onAction(element.action, {
              ...element,
              checked,
            });
          }
        }}
      />
    </div>
  );
}
