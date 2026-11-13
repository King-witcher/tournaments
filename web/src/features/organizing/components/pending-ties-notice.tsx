import { useTranslations } from "next-intl";

import { OPEN_TIES_QUERY } from "@/features/matches/lib/ties";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { TaskNotice } from "@/shared/ui/task-notice";
import type { Severity } from "../lib/severity";

/**
 * Um torneio cujas séries acabaram com empate na classificação esperando a
 * organização: só o título e o botão que abre a decisão deles na
 * Classificação.
 */
export function PendingTiesNotice({
  kind,
  item,
}: {
  kind: Severity;
  item: components["schemas"]["PendingTies"];
}) {
  const t = useTranslations("Organizing.pendingTies");
  const { tournament } = item;

  return (
    <TaskNotice
      kind={kind}
      title={t("title")}
      action={
        <ButtonLink
          href={`/${tournament.tenant_slug}/${tournament.slug}/standings?${OPEN_TIES_QUERY}`}
          kind="ghost"
          size="sm"
        >
          {t("action")}
        </ButtonLink>
      }
    />
  );
}
