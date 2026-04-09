import { NoPluginLayout } from "@/bindings/component/layout/NoPluginLayout";
import { PageWrapper } from "@/components/layout/PageLayout";

export default function LogsPage() {
  return (
    <PageWrapper title="History">
      <NoPluginLayout>
        <div>Welcome to the History page</div>
      </NoPluginLayout>
    </PageWrapper>
  );
}
