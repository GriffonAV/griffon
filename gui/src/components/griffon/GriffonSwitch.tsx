import { Switch } from "@/components/ui/switch";
import type { GriffonActionHandler, SwitchElement } from "@/components/types";

type Props = {
  element: SwitchElement;
  onAction?: GriffonActionHandler;
};

export default function GriffonSwitch({ element, onAction }: Props) {
  return (
    <div
      id={element.id}
      className="flex items-center justify-between gap-4 rounded-lg border p-3"
    >
      <div className="flex flex-col gap-1">
        <label className="text-sm font-medium leading-none">{element.label}</label>
        {element.description ? (
          <p className="text-xs text-muted-foreground">{element.description}</p>
        ) : null}
      </div>

      <Switch
        defaultChecked={!!element.checked}
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