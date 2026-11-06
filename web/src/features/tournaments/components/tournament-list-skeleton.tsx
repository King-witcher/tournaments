import { useTranslations } from "next-intl";

import {
  SectionSkeleton,
  TabLoading,
  TableSkeleton,
} from "@/shared/ui/skeleton";

/**
 * A aba Torneios da comunidade carregando, na forma dela: os ativos em tiles
 * e a tabela dos encerrados (`TournamentList`).
 */
export function TournamentListSkeleton() {
  const t = useTranslations("Tournaments");
  return (
    <TabLoading label={t("loading")}>
      <SectionSkeleton tiles={3} />
      <TableSkeleton columns={4} />
    </TabLoading>
  );
}
