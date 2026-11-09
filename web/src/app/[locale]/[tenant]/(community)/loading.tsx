import { getTranslations } from "next-intl/server";

import { TournamentListSkeleton } from "@/features/tournaments/components/tournament-list-skeleton";
import { PageColumns } from "@/shared/ui/page-columns";
import { PageHeaderSkeleton, SectionSkeleton } from "@/shared/ui/skeleton";

/** O início da comunidade carregando: o cabeçalho, os torneios e a coluna estreita. */
export default async function Loading() {
  const t = await getTranslations("Tournaments");
  return (
    <>
      <PageHeaderSkeleton breadcrumb={false} label={t("loading")} />
      <PageColumns
        main={<TournamentListSkeleton />}
        aside={<SectionSkeleton rows={3} />}
      />
    </>
  );
}
