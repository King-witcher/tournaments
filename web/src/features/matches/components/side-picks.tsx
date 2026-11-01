import { useTranslations } from "next-intl";

import { ChampionPortrait } from "@/features/champions/components/champion-portrait";
import { shownNickname } from "@/features/members/lib/nickname";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { StatusTag } from "@/shared/ui/status-tag";
import { ChampionFigure } from "./champion-figure";
import styles from "./side-picks.module.scss";

type Schemas = components["schemas"];

type Props = {
  veto: Schemas["VetoView"];
  game: Schemas["VetoGame"];
  side: Schemas["Side"];
  viewerId: number | null;
  /** `md`: o resumo do draft; `lg`: o cartão da partida. */
  size?: "sm" | "md" | "lg";
};

/**
 * O campeão de um lado numa partida: no duelo, o retrato e o nome (ou o
 * quadro vazio, a escolher); no TDM, um por jogador, com quem ainda escolhe
 * na janela, e quem vê marcado como "Você".
 */
export function SidePicks({ veto, game, side, viewerId, size = "md" }: Props) {
  const t = useTranslations("Match.veto.draft");
  const tMember = useTranslations("Member");
  if (veto.team_pick_seconds == null) {
    return (
      <ChampionFigure
        champion={game[`pick_${side}`]}
        empty={t("toPick")}
        size={size}
      />
    );
  }
  const nick = (n: string) => shownNickname(n, tMember("deletedAccount"));
  const players = [
    ...game.team_picks
      .filter((p) => p.side === side)
      .map((p) => ({
        id: p.user_id,
        nickname: p.nickname,
        champion: p.champion,
        drawn: p.timeout,
      })),
    ...veto.missing
      .filter((p) => p.side === side && p.games.includes(game.game))
      .map((p) => ({
        id: p.user_id,
        nickname: p.nickname,
        champion: null,
        drawn: false,
      })),
  ];
  if (players.length === 0) {
    return <ChampionFigure champion={null} empty={t("toPick")} size="sm" />;
  }
  return (
    <ul className={cn(styles.players, styles[size])}>
      {players.map((p) => (
        <li
          key={p.id}
          className={cn(styles.player, p.id === viewerId && styles.you)}
        >
          <span className={styles.portrait}>
            {p.champion ? (
              <ChampionPortrait slug={p.champion.slug} />
            ) : (
              <span aria-hidden className={styles.placeholder} />
            )}
          </span>
          <div className={styles.text}>
            <div className={styles.nick}>
              {nick(p.nickname)}
              {p.id === viewerId && (
                <StatusTag status="neutral" size="sm">
                  {t("you")}
                </StatusTag>
              )}
            </div>
            <span
              className={cn(styles.champion, !p.champion && styles.choosing)}
            >
              {p.champion
                ? p.drawn
                  ? t("drawnChampion", { champion: p.champion.name })
                  : p.champion.name
                : t("choosing")}
            </span>
          </div>
        </li>
      ))}
    </ul>
  );
}
