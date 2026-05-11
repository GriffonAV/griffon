import { textAlignClass, textVariantClass, toneTextClass } from "@/components/utils";
import type { TextAlign, TextVariant, Tone } from "@/components/types";
import { resolveFromPath } from "@/lib/utils";

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


function resolveTemplate(value: string, context: { store: any; event?: any }) {
  return value.replace(/\{\{(.*?)\}\}/g, (_, rawKey) => {
    const path = String(rawKey).trim();
    const resolved = resolveFromPath(path, context);
    return resolved !== undefined && resolved !== null ? String(resolved) : "";
  });
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
    <div
      className={[
        textVariantClass(element.variant),
        toneTextClass(element.tone),
        textAlignClass(element.align),
      ].join(" ")}
    >
      {content}
    </div>
  );
}