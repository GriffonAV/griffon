import { ChangeThemeButtonTest, ModeToggle, ModeToggleGroup } from "@/components/layout/ModeToggle";
import { PageWrapper } from "@/components/layout/PageLayout";

import {
  NavigationMenu,
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuTrigger,
  navigationMenuTriggerStyle,
} from "@/components/ui/navigation-menu"

export default function SettingsPage() {
  return (
    <PageWrapper title="Settings">
      <div className="flex">
        <NavigationMenu className="h-min">
          <NavigationMenuList>
            <NavigationMenuItem>
              <NavigationMenuLink asChild className={navigationMenuTriggerStyle()}>
                <span>Appearance</span>
              </NavigationMenuLink>
            </NavigationMenuItem>
            <NavigationMenuItem>
              <NavigationMenuLink asChild className={navigationMenuTriggerStyle()}>
                <span>Notifications</span>
              </NavigationMenuLink>
            </NavigationMenuItem>
          </NavigationMenuList>
        </NavigationMenu>
      </div>
      <div>
        <h2 className="text-xl font-bold">Appearance</h2>
        <div className="flex flex-col m-5">
          <p>
            Choose theme: <ModeToggleGroup />
          </p>
        </div>
        <ChangeThemeButtonTest />
      </div>
      <div>
        <h2 className="text-xl font-bold p-2">Notifications</h2>
        <p></p>

      </div>
    </PageWrapper >
  );
}
