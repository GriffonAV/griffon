import GriffonElementRenderer from "@/renderer/GriffonElementRenderer";

interface GriffonSectionRendererProps {
  section: {
    id: string;
    contents: any[];
  };
  store?: Record<string, any>;
  onAction?: (action: string, event?: any) => void;
}

export default function GriffonSectionRenderer({
  section,
  store = {},
  onAction,
}: GriffonSectionRendererProps) {
  return (
    <div className="flex flex-col gap-4">
      {section.contents?.map((element) => (
        <GriffonElementRenderer
          key={element.id}
          element={element}
          store={store}
          onAction={onAction}
        />
      ))}
    </div>
  );
}