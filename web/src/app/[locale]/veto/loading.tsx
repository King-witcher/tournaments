import { Page } from "@/shared/ui/page";
import { FormSkeleton, PageHeaderSkeleton } from "@/shared/ui/skeleton";

/** O formulário de criar uma sala carregando. */
export default function Loading() {
  return (
    <Page>
      <PageHeaderSkeleton breadcrumb={false} />
      <FormSkeleton label={false} />
    </Page>
  );
}
