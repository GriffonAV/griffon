import type { GriffonActionHandler, GroupElement } from "@/components/types";
import { gapClass } from "@/components/utils";
import GriffonElementRenderer from "@/renderer/GriffonElementRenderer";

type Props = {
  element: GroupElement;
  onAction?: GriffonActionHandler;
};

export default function GriffonGroup({ element, onAction }: Props) {
  return (
    <div id={element.id} className={["flex flex-col", gapClass(element.gap)].join(" ")}>
      {element.title ? (
        <div className="text-lg font-semibold tracking-tight">{element.title}</div>
      ) : null}

      {element.description ? (
        <div className="text-sm text-muted-foreground">{element.description}</div>
      ) : null}

      {element.children?.map((child, index) => (
        <GriffonElementRenderer
          key={child.id ?? `${element.id ?? "group"}-${index}`}
          element={child}
          onAction={onAction}
        />
      ))}
    </div>
  );
}