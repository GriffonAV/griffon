import React, { useState } from "react";
import { Button } from "../ui/button";
import { PanelLeft } from "lucide-react";
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuList,
} from "@/components/ui/navigation-menu";

interface PageProps {
  title?: string;
  children?: React.ReactNode[];
  navigation?: boolean;
  tabs?: string[];
}

export const PageWrapper2: React.FC<PageProps> = ({ title, children, navigation, tabs }) => {
  const [activeTab, setActiveTab] = useState<number>(0);

  return (
    <div className="bg-background text-foreground flex-1 flex-col m-2 rounded-md overflow-hidden flex">
      <div className="flex items-center border-b rounded-none p-2 gap-4">
        <Button className="cursor-pointer" variant={"ghost"} id="titlebar-maximize" title="maximize">
          <PanelLeft />
        </Button>
        <div className="border-r rounded-none w-0 h-6"></div>
        {title && <h1>{title}</h1>}
      </div>
      {navigation && tabs && (
        <div className="flex p-2 shadow">
          <NavigationMenu className="h-min">
            <NavigationMenuList>
              {tabs.map((tab, index) => (
                <NavigationMenuItem key={tab} className="cursor-pointer">
                  <Button
                    variant={activeTab === index ? "default" : "ghost"}
                    onClick={() => setActiveTab(index)}
                  >
                    {tab}
                  </Button>
                </NavigationMenuItem>
              ))}
            </NavigationMenuList>
          </NavigationMenu>
        </div>
      )}
      <main className="flex flex-col m-2 flex-1 overflow-auto sm:px-0 md:px-7 lg:px-36 items-start gap-4">
        {children?.[activeTab]}
      </main>
    </div>
  );
};
