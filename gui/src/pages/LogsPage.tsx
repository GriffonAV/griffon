import { NoPluginLayout } from "@/bindings/component/layout/NoPluginLayout";
import { PageLayout } from "@/components/layout/PageLayout";

export default function LogsPage() {
  return (
    <PageLayout title="History">
      <NoPluginLayout>
        <div>Welcome to the History page</div>
      </NoPluginLayout>
    </PageLayout>
  );
}
