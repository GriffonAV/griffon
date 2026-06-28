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
      <div className="flex flex-row items-center py-2">
        <span className="font-bold text-2xl">{section.id.charAt(0).toUpperCase() + section.id.slice(1).replace(/_/g, ' ')}</span>
      </div>
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