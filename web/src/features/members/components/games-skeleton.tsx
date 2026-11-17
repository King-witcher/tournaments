import { useTranslations } from "next-intl";

import { DropdownSkeleton } from "@/shared/ui/carbon/dropdown";
import { FormRow } from "@/shared/ui/form-row";
import {
  SectionSkeleton,
  TabLoading,
  TableSkeleton,
} from "@/shared/ui/skeleton";

/** A aba Partidas carregando: a frase, os três filtros e a tabela. */
export function GamesSkeleton() {
  const t = useTranslations("Member.loadingTab");
  return (
    <TabLoading label={t("games")}>
      <SectionSkeleton rows={0} />
      <div aria-hidden>
        <FormRow columns={3}>
          <DropdownSkeleton />
          <DropdownSkeleton />
          <DropdownSkeleton />
        </FormRow>
      </div>
      <TableSkeleton columns={8} rows={10} />
    </TabLoading>
  );
}
