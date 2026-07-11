import { textAlignClass, textVariantClass, toneTextClass } from "@/components/utils";
import type { TextAlign, TextVariant, Tone } from "@/components/types";
import { resolveTemplate } from "@/lib/utils";

interface GriffonTextProps {
  element: {
    id: string;
    name?: string;
    variant?: TextVariant;
    tone?: Tone;
    align?: TextAlign;
    [key: string]: any;
  };
  store?: Record<string, any>;
}


export default function GriffonText({
  element,
  store = {},
}: GriffonTextProps) {
  const content =
    typeof element.name === "string"
      ? resolveTemplate(element.name, { store })
      : element.name ?? "";

  return (
    <div>
      {content &&
        <div
          className={[
            textVariantClass(element.variant),
            toneTextClass(element.tone),
            textAlignClass(element.align),
          ].join(" ")}
        >
          {content}
        </div>
      }
    </div>
  );
}