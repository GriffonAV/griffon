import { NoPluginLayout } from "@/bindings/component/layout/NoPluginLayout";
import { PageWrapper } from "@/components/layout/PageLayout";

export default function HomePage() {
  return (
    <PageWrapper title="Home">
      <NoPluginLayout>
        <div>Welcome Home</div>
      </NoPluginLayout>
    </PageWrapper>
  );
}
