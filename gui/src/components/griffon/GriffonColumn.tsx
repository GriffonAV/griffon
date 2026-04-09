import type { ColumnElement, GriffonActionHandler } from "../types";
import { alignItemsClass, gapClass, justifyClass } from "../utils";
import GriffonElementRenderer from "@/renderer/GriffonElementRenderer";

type Props = {
  element: ColumnElement;
  onAction?: GriffonActionHandler;
};

export default function GriffonColumn({ element, onAction }: Props) {
  return (
    <div
      id={element.id}
      className={[
        "flex flex-col",
        gapClass(element.gap),
        alignItemsClass(element.align),
        justifyClass(element.justify),
      ].join(" ")}
    >
      {element.children?.map((child, index) => (
        <GriffonElementRenderer
          key={child.id ?? `${element.id ?? "column"}-${index}`}
          element={child}
          onAction={onAction}
        />
      ))}
    </div>
  );
}