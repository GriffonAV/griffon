import { NoPluginLayout } from "@/bindings/component/layout/NoPluginLayout";
import { PageLayout } from "@/components/layout/PageLayout";

export default function LogsPage() {
  return (
    <PageLayout title="Activity Log">
      <NoPluginLayout>
        <div>Welcome to the Activity Log page</div>
      </NoPluginLayout>
    </PageLayout>
  );
}
