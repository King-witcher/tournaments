"use client";

import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { StatusTag } from "@/shared/ui/status-tag";
import { useLiveMatch } from "../hooks/use-live-match";
import { useSideNames } from "../hooks/use-side-names";
import styles from "./game-card.module.scss";
import { SidePicks } from "./side-picks";

type Props = {
  side: components["schemas"]["Side"];
  /** A partida do cartão. */
  number: number;
};

/**
 * Um lado no cartão da partida, na coluna dele (a mesma do formulário de
 * envio embaixo): o nome e o campeão (no TDM, o de cada jogador). O lado de
 * quem vê fica destacado e com "Você".
 */
export function GameCardSide({ side, number }: Props) {
  const t = useTranslations("Match.veto.draft");
  const { detail, viewerId } = useLiveMatch();
  const names = useSideNames(detail);
  const veto = detail.veto;
  const game = veto?.games.find((g) => g.game === number);
  const mine = detail.viewer_side === side;

  return (
    <div className={cn(styles.side, styles[side], mine && styles.mine)}>
      <div className={styles.who}>
        {names[side]}
        {mine && (
          <StatusTag status="neutral" size="sm">
            {t("you")}
          </StatusTag>
        )}
      </div>
      {veto?.champion_phase && game && (
        <SidePicks
          veto={veto}
          game={game}
          side={side}
          viewerId={viewerId}
          size="lg"
        />
      )}
    </div>
  );
}
