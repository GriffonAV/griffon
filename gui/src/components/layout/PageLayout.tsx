import React, { useState, useEffect, useRef } from "react";
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
  navigation?: boolean;
  same_page?: boolean;
  tabs?: string[];
}

export const PageWrapper: React.FC<PageProps> = ({ title, children, navigation, tabs }) => {
  const [activeSection, setActiveSection] = useState<string | null>(null);
  const sectionRefs = useRef<(HTMLElement | null)[]>([]);

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
                    variant={activeSection === tab ? "default" : "ghost"}
                    onClick={() => scrollToSection(index)}
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
        {React.Children.map(children, (child, index) => (
          <div
            id={tabs?.[index]}
            ref={(el) => (sectionRefs.current[index] = el)}
            className="w-full"
          >
            {child}
          </div>
        ))}
      </main>
    </div>
  );
};
