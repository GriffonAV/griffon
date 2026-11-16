import GriffonElementRenderer from "@/renderer/GriffonElementRenderer";

interface GriffonSectionRendererProps {
  section: {
    id: string;
    title?: string;
    description?: string;
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
  // console.log("Rendering GriffonSectionRenderer for section:", section.id);
  // console.log("Section contents:", section.contents);
  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1 py-2">
        <h2 className="text-2xl font-bold">
          {section.title ?? section.id.charAt(0).toUpperCase() + section.id.slice(1).replace(/_/g, ' ')}
        </h2>
        {section.description ? (
          <p className="max-w-3xl text-sm leading-relaxed text-muted-foreground">
            {section.description}
          </p>
        ) : null}
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
