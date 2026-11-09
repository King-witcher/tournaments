import { useLocale, useTranslations } from "next-intl";

import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { MetaLines } from "@/shared/ui/meta-lines";
import { TaskNotice } from "@/shared/ui/task-notice";
import type { Severity } from "../lib/severity";

/**
 * Um torneio com a fase seguinte pronta para iniciar (a anterior terminou):
 * a comunidade, a fase, desde quando espera e o botão que leva ao torneio,
 * onde a organização abre o check-in e inicia a fase.
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
      title={localizedName(tournament, locale)}
      meta={
        <MetaLines
          lines={[
            `${community ? `${community} · ` : ""}${t("title", { number: item.stage })}`,
            t.rich("since", {
              date: () => <LocalDateTime value={item.since} />,
            }),
          ]}
        />
      }
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
