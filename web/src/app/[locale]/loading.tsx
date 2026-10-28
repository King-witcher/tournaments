import { Page } from "@/shared/ui/page";
import { PageSkeleton } from "@/shared/ui/skeleton";

/**
 * Enquanto uma página sem `loading.tsx` próprio chega: o lugar do cabeçalho e
 * de uma seção. Cada rota que busca dado deve ter o seu, na forma dela.
 */
export default function Loading() {
  return (
    <Page>
      <PageSkeleton />
    </Page>
  );
}
