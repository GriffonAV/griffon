import type { GriffonActionHandler, RowElement } from "@/components/types";
import { alignItemsClass, gapClass, justifyClass } from "@/components/utils";
import GriffonElementRenderer from "@/renderer/GriffonElementRenderer";

type Props = {
  element: RowElement;
  onAction?: GriffonActionHandler;
};

export default function GriffonRow({ element, onAction }: Props) {
  return (
    <div
      id={element.id}
      className={[
        "flex flex-row",
        gapClass(element.gap),
        alignItemsClass(element.align),
        justifyClass(element.justify),
        element.wrap ? "flex-wrap" : "flex-nowrap",
      ].join(" ")}
    >
      {element.children?.map((child, index) => (
        <GriffonElementRenderer
          key={child.id ?? `${element.id ?? "row"}-${index}`}
          element={child}
          onAction={onAction}
        />
      ))}
    </div>
  );
}