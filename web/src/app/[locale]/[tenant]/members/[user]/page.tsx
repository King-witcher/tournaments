import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { MemberProfile } from "@/features/members/components/member-profile";
import { Page } from "@/shared/ui/page";
import { PageHeaderSkeleton, TableSkeleton } from "@/shared/ui/skeleton";

/**
 * A pessoa na comunidade. Fica fora do grupo das abas: é uma página própria,
 * com o próprio cabeçalho e a trilha até Membros.
 */
export default async function MemberPage({
  params,
}: PageProps<"/[locale]/[tenant]/members/[user]">) {
  const t = await getTranslations("Member");
  return (
    <Page width="wide">
      <Suspense
        fallback={
          <>
            <PageHeaderSkeleton meta label={t("loading")} />
            <TableSkeleton columns={4} rows={3} />
          </>
        }
      >
        {params.then(({ tenant, user }) => (
          <MemberProfile tenant={tenant} user={user} />
        ))}
      </Suspense>
    </Page>
  );
}
