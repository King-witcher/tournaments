import { Page } from "@/shared/ui/page";
import { PageSkeleton } from "@/shared/ui/skeleton";

/** A sala carregando. */
export default function Loading() {
  return (
    <Page width="wide">
      <PageSkeleton />
    </Page>
  );
}
