import { useLocale, useTranslations } from "next-intl";

import { OPEN_TIES_QUERY } from "@/features/matches/lib/ties";
import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { TaskNotice } from "@/shared/ui/task-notice";
import type { Severity } from "../lib/severity";

/**
 * Um torneio cujas séries acabaram com empate na classificação esperando a
 * organização: a comunidade e o botão que abre a decisão deles na
 * Classificação.
 */
export function PendingTiesNotice({
  kind,
  item,
  community,
}: {
  kind: Severity;
  item: components["schemas"]["PendingTies"];
  /** O nome da comunidade, quando a lista é de várias. */
  community?: string;
}) {
  const t = useTranslations("Organizing.pendingTies");
  const locale = useLocale();
  const { tournament } = item;

  return (
    <TaskNotice
      kind={kind}
      title={localizedName(tournament, locale)}
      meta={`${community ? `${community} · ` : ""}${t("title")}`}
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
