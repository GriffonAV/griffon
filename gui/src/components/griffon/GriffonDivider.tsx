import { Separator } from "@/components/ui/separator";
import type { DividerElement } from "@/components/types";

type Props = {
  element: DividerElement;
};

export default function GriffonDivider({ element }: Props) {
  const orientation = element.orientation === "vertical" ? "vertical" : "horizontal";

  if (element.label) {
    return (
      <div id={element.id} className="flex items-center gap-3 w-full">
        <Separator className="flex-1" />
        <span className="text-xs uppercase tracking-wide text-muted-foreground">
          {element.label}
        </span>
        <Separator className="flex-1" />
      </div>
    );
  }

  return <Separator id={element.id} orientation={orientation} />;
}