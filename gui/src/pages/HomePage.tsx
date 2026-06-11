import { NoPluginLayout } from "@/bindings/component/layout/NoPluginLayout";
import { PageLayout } from "@/components/layout/PageLayout";

export default function HomePage() {
  return (
    <PageLayout title="Home">
      <NoPluginLayout>
        <div>Welcome Home</div>
      </NoPluginLayout>
    </PageLayout>
  );
}
