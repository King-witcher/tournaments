import { useTranslations } from "next-intl";

import { ActionTile } from "@/shared/ui/action-tile";
import { ButtonLink } from "@/shared/ui/button-link";
import { TileGrid } from "@/shared/ui/tile-grid";

/**
 * Na Visão geral, para a organização: há empates do fim da fase a decidir,
 * com o botão que leva à Classificação, onde se decide a ordem.
 */
export function PendingTiesTask({ href }: { href: string }) {
  const t = useTranslations("Bracket.ties");
  const tOrganizing = useTranslations("Organizing.pendingTies");
  return (
    <TileGrid stacked>
      <ActionTile
        title={t("title")}
        meta={t("description")}
        action={
          <ButtonLink href={href} kind="tertiary" size="sm">
            {tOrganizing("action")}
          </ButtonLink>
        }
      />
    </TileGrid>
  );
}
