import { Page } from "@/shared/ui/page";
import { PageSkeleton } from "@/shared/ui/skeleton";

/** Enquanto a página do segmento chega: o lugar do título e do conteúdo. */
export default function Loading() {
  return (
    <Page spacing="loose">
      <PageSkeleton />
    </Page>
  );
}
