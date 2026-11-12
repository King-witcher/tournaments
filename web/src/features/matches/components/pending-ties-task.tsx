import { useTranslations } from "next-intl";

import { severity } from "@/features/organizing/lib/severity";
import { ButtonLink } from "@/shared/ui/button-link";
import { TaskNotice } from "@/shared/ui/task-notice";

/**
 * Na Visão geral, para a organização: há empates do fim da fase a decidir,
 * com o botão que abre a decisão deles na Classificação.
 */
export function PendingTiesTask({
  href,
  stage,
  lastStage,
}: {
  href: string;
  /** A fase dos empates. */
  stage: number;
  /** É a última fase: decididos, o torneio termina (senão, a fase). */
  lastStage: boolean;
}) {
  const t = useTranslations("Organizing.pendingTies");
  return (
    <TaskNotice
      kind={severity("pending_ties")}
      title={lastStage ? t("title") : t("titleStage", { stage })}
      meta={t(lastStage ? "meta" : "metaStage")}
      action={
        <ButtonLink href={href} kind="ghost" size="sm">
          {t("action")}
        </ButtonLink>
      }
    />
  );
}
