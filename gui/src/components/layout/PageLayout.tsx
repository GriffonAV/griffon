import React, { useState, useEffect, useRef } from "react";
import { Button } from "../ui/button";
import { PanelLeft } from "lucide-react";
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuList,
} from "@/components/ui/navigation-menu";
import { titleCase } from "@/lib/titleCase";
import { useSidebar } from "@/providers/SidebarProvider";

interface PageProps {
  title?: string;
  children?: React.ReactNode;
  navigation?: boolean;
  same_page?: boolean;
  tabs?: string[];
}

export const PageWrapper: React.FC<PageProps> = ({ title, children, navigation, tabs }) => {
  const [activeSection, setActiveSection] = useState<string | null>(null);
  const sectionRefs = useRef<(HTMLElement | null)[]>([]);
  const { toggleSidebar, isCollapsed } = useSidebar();

  // Function to scroll to a section
  const scrollToSection = (index: number) => {
    sectionRefs.current[index]?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setActiveSection(entry.target.id);
          }
        });
      },
      { threshold: 0.5 } // Adjust threshold as needed
    );

    sectionRefs.current.forEach((ref) => {
      if (ref) observer.observe(ref);
    });

    return () => {
      sectionRefs.current.forEach((ref) => {
        if (ref) observer.unobserve(ref);
      });
    };
  }, [children]);

  return (
    <div className="bg-background text-foreground flex-1 flex-col m-2 rounded-md overflow-hidden flex">
      <div className="flex items-center border-b rounded-none p-2 gap-2">
        <Button className="cursor-pointer" variant={"ghost"} id="titlebar-maximize" title="maximize" onClick={toggleSidebar}>
          {isCollapsed ? <PanelLeft /> : <PanelLeft />}
        </Button>
        <div className="border-r rounded-none w-0 h-6"></div>
        <h1 className="text-lg font-semibold pl-6">
          {title && titleCase(title)}
        </h1>
      </div>
      {navigation && tabs && (
        <div className="flex p-2 shadow">
          <NavigationMenu className="h-min">
            <NavigationMenuList>
              {tabs.map((tab, index) => (
                <NavigationMenuItem key={tab} className="cursor-pointer">
                  <Button
                    variant={activeSection === tab ? "default" : "ghost"}
                    onClick={() => scrollToSection(index)}
                  >
                    {titleCase(tab)}
                  </Button>
                </NavigationMenuItem>
              ))}
            </NavigationMenuList>
          </NavigationMenu>
        </div>
      )}
      <main className="flex flex-col m-2 mt-6 flex-1 overflow-auto sm:px-0 md:px-7 lg:px-36 items-start gap-4">
        {React.Children.map(children, (child, index) => (
          <div
            id={tabs?.[index]}
            ref={(el) => {
              sectionRefs.current[index] = el;
            }}
            className="w-full"
          >
            {child}
          </div>
        ))}
      </main>
    </div>
  );
};
