import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { MemberShell } from "@/features/members/components/member-shell";
import { LiveUpdates } from "@/features/realtime/components/live-updates";
import { Page } from "@/shared/ui/page";
import {
  PageHeaderSkeleton,
  SectionSkeleton,
  TabsSkeleton,
} from "@/shared/ui/skeleton";

/**
 * A pessoa na comunidade: o cabeçalho e as abas (Visão geral, Partidas,
 * Torneios, Confrontos), cada uma uma sub-rota com o próprio `loading.tsx`.
 * Fica fora do grupo das seções da comunidade: é uma página própria, com a
 * trilha até Membros.
 */
export default async function MemberLayout({
  params,
  children,
}: LayoutProps<"/[locale]/[tenant]/members/[user]">) {
  const t = await getTranslations("Member");
  return (
    <Page width="wide">
      <Suspense
        fallback={
          <>
            <PageHeaderSkeleton meta label={t("loading")} />
            <TabsSkeleton />
            <SectionSkeleton rows={4} />
          </>
        }
      >
        {params.then(({ tenant, user }) => (
          <>
            <LiveUpdates tenant={tenant} />
            <MemberShell tenant={tenant} user={user}>
              {children}
            </MemberShell>
          </>
        ))}
      </Suspense>
    </Page>
  );
}
