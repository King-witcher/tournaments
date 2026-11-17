import { useTranslations } from "next-intl";

import { Panel } from "@/shared/ui/panel";
import {
  SectionSkeleton,
  TabLoading,
  TableSkeleton,
} from "@/shared/ui/skeleton";

/** A aba Confrontos carregando: a modalidade, o destaque e a tabela. */
export function OpponentsSkeleton() {
  const t = useTranslations("Member.loadingTab");
  return (
    <TabLoading label={t("opponents")}>
      <SectionSkeleton rows={0} />
      <Panel>
        <SectionSkeleton rows={2} />
      </Panel>
      <TableSkeleton columns={4} rows={8} />
    </TabLoading>
  );
}
