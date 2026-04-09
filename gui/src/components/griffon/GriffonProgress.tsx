import { Progress } from "@/components/ui/progress";
import type { ProgressElement } from "@/components/types";

type Props = {
  element: ProgressElement;
};

export default function GriffonProgress({ element }: Props) {
  return (
    <div id={element.id} className="flex flex-col gap-2">
      {(element.label || element.show_value) ? (
        <div className="flex items-center justify-between gap-2">
          {element.label ? (
            <span className="text-sm font-medium">{element.label}</span>
          ) : (
            <span />
          )}

          {element.show_value ? (
            <span className="text-xs text-muted-foreground">
              {element.value}%
            </span>
          ) : null}
        </div>
      ) : null}

      <Progress value={element.value} />
    </div>
  );
}