import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { TaskNotice } from "@/shared/ui/task-notice";
import type { Severity } from "../lib/severity";

/**
 * Um torneio com a fase seguinte pronta para iniciar (a anterior terminou):
 * só a fase, no título, e o botão que leva ao torneio, onde a organização abre
 * o check-in e inicia a fase.
 */
export function StageDueNotice({
  kind,
  item,
}: {
  kind: Severity;
  item: components["schemas"]["StageDue"];
}) {
  const t = useTranslations("Organizing.stageDue");
  const { tournament } = item;

  return (
    <TaskNotice
      kind={kind}
      title={t("title", { number: item.stage })}
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
