import React from "react";
import { Button } from "../ui/button";
import { PanelLeft } from "lucide-react";

import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  navigationMenuTriggerStyle,
} from "@/components/ui/navigation-menu";
import { Separator } from "@/components/ui/separator";

interface PageProps {
  title?: string;
  children?: React.ReactNode;
  navigation?: Boolean;
  sections?: string[];
}

export const PageWrapper: React.FC<PageProps> = ({ title, children, navigation, sections }) => {
  return (
    <div className="bg-background text-foreground flex-1 flex-col m-2 rounded-md overflow-hidden flex">
      <div className="flex items-center border-b rounded-none p-2 gap-4">
        <Button
          className="cursor-pointer"
          variant={"ghost"}
          id="titlebar-maximize"
          title="maximize"
        >
          <PanelLeft />
        </Button>
        <div className="border-r rounded-none w-0 h-6"></div>
        {title && <h1>{title}</h1>}
      </div>
      {
        navigation &&
        <div className="flex p-2 shadow">
          <NavigationMenu className="h-min">
            <NavigationMenuList>
              {/* <NavigationMenuItem>
                <NavigationMenuLink asChild className={navigationMenuTriggerStyle()}>
                  <span>Appearance</span>
                </NavigationMenuLink>
              </NavigationMenuItem> */}
              {sections?.map((section) => (
                <NavigationMenuItem >
                  <NavigationMenuLink asChild className={navigationMenuTriggerStyle()}>
                    <span>{section} </span>
                  </NavigationMenuLink>
                </NavigationMenuItem>
              ))
              }
            </NavigationMenuList>
          </NavigationMenu>
        </div>
      }
      <main className="flex flex-col m-2 flex-1 overflow-auto sm:px-0 md:px-7 lg:px-36 items-start gap-4">
        {children}
      </main>
    </div >
  );
};
