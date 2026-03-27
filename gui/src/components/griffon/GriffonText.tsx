import type { TextElement } from "@/components/types";
import { textAlignClass, textVariantClass, toneTextClass } from "@/components/utils";

type Props = {
  element: TextElement;
};

export default function Text({ element }: Props) {
  return (
    <div
      id={element.id}
      className={[
        textVariantClass(element.variant),
        toneTextClass(element.tone),
        textAlignClass(element.align),
      ].join(" ")}
    >
      {element.name}
    </div>
  );
}