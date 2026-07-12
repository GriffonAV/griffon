import React, { useState, useEffect, useRef } from "react";
import { Button } from "../ui/button";
import { Columns2, PanelLeft } from "lucide-react";
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
  tabs?: string[];
  mode?: "scroll" | "tabs"; // New prop to determine the behavior
}

export const PageLayout: React.FC<PageProps> = ({
  title,
  children,
  navigation,
  tabs,
  mode = "scroll" // Defaults to scroll behavior if not specified
}) => {
  // State for both modes
  const [activeSection, setActiveSection] = useState<string | null>(tabs?.[0] || null);
  const [activeTab, setActiveTab] = useState<number>(0);

  const sectionRefs = useRef<(HTMLElement | null)[]>([]);
  const mainRef = useRef<HTMLElement>(null); // <-- Add this new ref
  const { toggleSidebar, isCollapsed } = useSidebar();

  // Scroll logic for "scroll" mode
  const scrollToSection = (index: number) => {
    sectionRefs.current[index]?.scrollIntoView({ behavior: "smooth" });
  };



  useEffect(() => {
    if (mode !== "scroll") return;

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setActiveSection(entry.target.id);
          }
        });
      },
      {
        root: mainRef.current, // Calculate intersections relative to the scrolling main container
        rootMargin: "0px 0px -50% 0px", // Trigger when a section enters the top 50% of the view
        threshold: 0 // Trigger as soon as the element hits the margin, regardless of its height
      }
    );

    sectionRefs.current.forEach((ref) => {
      if (ref) observer.observe(ref);
    });

    return () => {
      sectionRefs.current.forEach((ref) => {
        if (ref) observer.unobserve(ref);
      });
    };
  }, [children, mode]);

  return (
    <div className="bg-background text-foreground flex-1 flex-col m-2 rounded-md overflow-hidden flex">
      {/* Header */}
      <div className="flex items-center border-b rounded-none p-2 gap-2">
        <Button className="cursor-pointer" variant={"ghost"} id="titlebar-maximize" title="maximize" onClick={toggleSidebar}>
          {isCollapsed ? <Columns2 /> : <PanelLeft />}
        </Button>
        <div className="border-r rounded-none w-0 h-6"></div>
        <h1 className="text-lg font-semibold pl-6">
          {title && titleCase(title)}
        </h1>
      </div>

      {/* Navigation */}
      {navigation && tabs && (
        <div className="flex p-2 shadow">
          <NavigationMenu className="h-min">
            <NavigationMenuList>
              {tabs.map((tab, index) => {
                const isActive = mode === "scroll"
                  ? activeSection === tab
                  : activeTab === index;

                return (
                  <NavigationMenuItem key={tab} className="cursor-pointer">
                    <Button
                      variant={isActive ? "default" : "ghost"}
                      onClick={() => {
                        if (mode === "scroll") {
                          scrollToSection(index);
                        } else {
                          setActiveTab(index);
                        }
                      }}
                    >
                      {titleCase(tab)}
                    </Button>
                  </NavigationMenuItem>
                );
              })}
            </NavigationMenuList>
          </NavigationMenu>
        </div>
      )}

      {/* Main Content Area */}
      <main
        ref={mainRef}
        className={`flex flex-col m-2 mt-6 flex-1 overflow-auto sm:px-0 md:px-7 lg:px-36 gap-4 ${mode === "scroll" ? "items-start pb-[50vh]" : "items-center"}`}
      >
        {mode === "scroll" ? (
          // Render all children, wrapped in refs for scrolling
          React.Children.map(children, (child, index) => (
            <div
              id={tabs?.[index]}
              ref={(el) => {
                sectionRefs.current[index] = el;
              }}
              className="w-full"
            >
              {child}
            </div>
          ))
        ) : (
          // Render only the active child for tab mode
          React.Children.toArray(children)[activeTab]
        )}
      </main>
    </div>
  );
};