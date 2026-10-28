import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { TenantFrame } from "@/features/tenants/components/tenant-frame";
import { Page } from "@/shared/ui/page";
import {
  PageHeaderSkeleton,
  TableSkeleton,
  TabsSkeleton,
} from "@/shared/ui/skeleton";

/**
 * A comunidade: o cabeçalho e as abas (Torneios, Membros, Convites,
 * Configurações), com a aba aberta dentro. Novo torneio e as páginas de
 * torneio ficam fora do grupo, sem as abas.
 */
export default async function CommunityLayout({
  params,
  children,
}: LayoutProps<"/[locale]/[tenant]">) {
  const t = await getTranslations("Tenant");
  return (
    <Page width="wide">
      <Suspense
        fallback={
          <>
            <PageHeaderSkeleton label={t("loading")} />
            <TabsSkeleton />
            <TableSkeleton columns={4} />
          </>
        }
      >
        {params.then(({ tenant }) => (
          <TenantFrame slug={tenant}>{children}</TenantFrame>
        ))}
      </Suspense>
    </Page>
  );
}
