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
  lastStage,
}: {
  href: string;
  /** É a última fase: decididos, o torneio termina (senão, a fase). */
  lastStage: boolean;
}) {
  const t = useTranslations("Bracket.ties");
  const tOrganizing = useTranslations("Organizing.pendingTies");
  return (
    <TaskNotice
      kind={severity("pending_ties")}
      title={t("title")}
      meta={t(lastStage ? "description" : "descriptionStage")}
      action={
        <ButtonLink href={href} kind="ghost" size="sm">
          {tOrganizing("action")}
        </ButtonLink>
      }
    />
  );
}
