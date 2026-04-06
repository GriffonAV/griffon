import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";
import type { GriffonActionHandler, TabsElement } from "@/components/types";
import GriffonElementRenderer from "@/renderer/GriffonElementRenderer";

type Props = {
  element: TabsElement;
  onAction?: GriffonActionHandler;
};

export default function GriffonTabs({ element, onAction }: Props) {
  const defaultTab = element.tabs?.[0]?.value;

  return (
    <div id={element.id} className="flex flex-col gap-4">
      <Tabs
        defaultValue={element.value ?? defaultTab}
        onValueChange={(value) => {
          if (element.action && onAction) {
            onAction(element.action, {
              ...element,
              value,
            });
          }
        }}
      >
        <TabsList className="w-full flex flex-wrap">
          {element.tabs?.map((tab) => (
            <TabsTrigger key={tab.value} value={tab.value}>
              {tab.label}
            </TabsTrigger>
          ))}
        </TabsList>

        {element.tabs?.map((tab) => (
          <TabsContent key={tab.value} value={tab.value} className="space-y-4">
            {tab.children?.map((child, index) => (
              <GriffonElementRenderer
                key={child.id ?? `${tab.value}-${index}`}
                element={child}
                onAction={onAction}
              />
            ))}
          </TabsContent>
        ))}
      </Tabs>
    </div>
  );
}