import type { GriffonSection, GriffonActionHandler } from "@/components/types";
import GriffonElementRenderer from "./GriffonElementRenderer";

type Props = {
  section: GriffonSection;
  onAction?: GriffonActionHandler;
};

export default function GriffonSectionRenderer({ section, onAction }: Props) {
  return (
    <div className="flex flex-col gap-4">
      {section.contents?.map((element, index) => (
        <GriffonElementRenderer
          key={element.id ?? `${section.id}-${index}`}
          element={element}
          onAction={onAction}
        />
      ))}
    </div>
  );
}