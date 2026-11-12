import { useLocale, useTranslations } from "next-intl";

import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { TaskNotice } from "@/shared/ui/task-notice";
import type { Severity } from "../lib/severity";

/**
 * Um torneio com a fase seguinte pronta para iniciar (a anterior terminou):
 * a fase no título, a comunidade e o torneio embaixo e o botão que leva ao
 * torneio, onde a organização abre o check-in e inicia a fase.
 */
export function StageDueNotice({
  kind,
  item,
  community,
}: {
  kind: Severity;
  item: components["schemas"]["StageDue"];
  /** O nome da comunidade, quando a lista é de várias. */
  community?: string;
}) {
  const t = useTranslations("Organizing.stageDue");
  const locale = useLocale();
  const { tournament } = item;

  return (
    <TaskNotice
      kind={kind}
      title={t("title", { number: item.stage })}
      meta={`${community ? `${community} · ` : ""}${localizedName(tournament, locale)}`}
      action={
        <ButtonLink
          href={`/${tournament.tenant_slug}/${tournament.slug}`}
          kind="ghost"
          size="sm"
        >
          {t("action")}
        </ButtonLink>
      }
    />
  );
}
