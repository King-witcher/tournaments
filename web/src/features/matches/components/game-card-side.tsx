"use client";

import { useTranslations } from "next-intl";

import { shownNickname } from "@/features/members/lib/nickname";
import { teamLeft } from "@/features/teams/lib/team-name";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { StatusTag } from "@/shared/ui/status-tag";
import { useLiveMatch } from "../hooks/use-live-match";
import { useSideNames } from "../hooks/use-side-names";
import { DepartureTag } from "./departure-tag";
import styles from "./game-card-side.module.scss";
import { SidePicks } from "./side-picks";

type Props = {
  side: components["schemas"]["Side"];
  /** A partida do cartão. */
  number: number;
};

/**
 * Um lado no cartão da partida, na coluna dele (a mesma do placar embaixo):
 * o nome, com "Você" no de quem vê e a marca de quem saiu, e o campeão (no
 * TDM, o de cada jogador; sem draft, os jogadores do time, com o capitão marcado).
 */
export function GameCardSide({ side, number }: Props) {
  const t = useTranslations("Match.veto.draft");
  const tMember = useTranslations("Member");
  const { detail, viewerId } = useLiveMatch();
  const names = useSideNames(detail);
  const team = detail[side];
  const veto = detail.veto;
  const game = veto?.games.find((g) => g.game === number);
  const mine = detail.viewer_side === side;
  const picks = veto?.champion_phase && game;

  return (
    <div className={cn(styles.side, styles[side])}>
      <div className={styles.who}>
        <span className={styles.name}>{names[side]}</span>
        {mine && (
          <StatusTag status="neutral" size="sm">
            {t("you")}
          </StatusTag>
        )}
        {team && (
          <DepartureTag withdrawn={team.withdrawn} left={teamLeft(team)} />
        )}
      </div>
      {picks && (
        <SidePicks
          veto={veto}
          game={game}
          side={side}
          viewerId={viewerId}
          size="lg"
        />
      )}
      {!picks && team && team.members.length > 1 && (
        <ul className={styles.members}>
          {team.members.map((m) => (
            <li key={m.user_id}>
              {shownNickname(m.nickname, tMember("deletedAccount"))}
              {m.user_id === team.captain && (
                <>
                  {" "}
                  <StatusTag status="neutral" size="sm">
                    {t("captain")}
                  </StatusTag>
                </>
              )}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
