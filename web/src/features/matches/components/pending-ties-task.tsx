import { useTranslations } from "next-intl";

import { ActionTile } from "@/shared/ui/action-tile";
import { ButtonLink } from "@/shared/ui/button-link";
import { TileGrid } from "@/shared/ui/tile-grid";

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
    <TileGrid stacked>
      <ActionTile
        title={t("title")}
        meta={t(lastStage ? "description" : "descriptionStage")}
        action={
          <ButtonLink href={href} kind="tertiary" size="sm">
            {tOrganizing("action")}
          </ButtonLink>
        }
      />
    </TileGrid>
  );
}
