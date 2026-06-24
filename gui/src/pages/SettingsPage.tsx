import { ChangeThemeButtonTest, ModeToggleGroup } from "@/components/layout/ModeToggle";
import { PageLayout } from "@/components/layout/PageLayout";
import { Badge } from "@/components/ui/badge";


export default function SettingsPage() {
  return (
    <PageLayout title="Settings" navigation tabs={["Background Service", "Appearance", "Notifications", "About"]}>
      <div title="Background Service">
        <h2 className="text-xl font-bold">Background Service</h2>

      </div>
      <div title="Appearance">
        <h2 className="text-xl font-bold">Appearance</h2>
        <div className="flex flex-col m-5">
          <p>
            Choose theme: <ModeToggleGroup />
          </p>
        </div>
        <ChangeThemeButtonTest />
      </div>
      <div title="Notifications">
        <h2 className="text-xl font-bold p-2">Notifications</h2>
        <p>      lorem ipsum, lorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsum
          lorem ipsum, lorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsumlorem ipsum</p>

      </div>
      <div title="About">
        <h2 className="text-xl font-bold p-2">About</h2>
        <p>


          You are using Griffon in version <Badge >0.3.0-alpha
          </Badge>.
        </p>

      </div>


    </PageLayout >
  );
}
