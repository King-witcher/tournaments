import { useTranslations } from "next-intl";

import { ChampionPortrait } from "@/features/champions/components/champion-portrait";
import { shownNickname } from "@/features/members/lib/nickname";
import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";
import { StatusTag } from "@/shared/ui/status-tag";
import { perPlayer } from "../lib/veto";
import { ChampionFigure } from "./champion-figure";
import styles from "./side-picks.module.scss";

type Schemas = components["schemas"];

type Props = {
  veto: Schemas["VetoView"];
  game: Schemas["VetoGame"];
  side: Schemas["Side"];
  viewerId: number | null;
  /**
   * `sm`: as partidas que faltam; `md`: o resumo do draft; `lg`: o cartão da
   * partida, com o retrato quadrado (`big`), por lado e por jogador.
   */
  size?: "sm" | "md" | "lg";
};

/**
 * O campeão de um lado numa partida, só o retrato (o nome fica para o leitor de tela): no
 * duelo alternado, um (ou o quadro vazio, a escolher); no pick por jogador, um por
 * jogador, com quem está escolhendo agora, quem falta e quem já escolheu às cegas
 * (sem dizer o quê), e quem vê marcado como "Você".
 */
export function SidePicks({ veto, game, side, viewerId, size = "md" }: Props) {
  const t = useTranslations("Match.veto.draft");
  const tMember = useTranslations("Member");
  if (!perPlayer(veto)) {
    return (
      <ChampionFigure
        champion={game[`pick_${side}`]}
        empty={t("toPick")}
        size={size}
      />
    );
  }
  const nick = (n: string) => shownNickname(n, tMember("deletedAccount"));
  // Nos picks alternados, só está escolhendo o lado da vez, na partida da vez.
  const turn = veto.turn;
  const choosing =
    turn?.action === "team_pick" &&
    (turn.pick_for == null ||
      (turn.pick_for === game.game && turn.sides.includes(side)));
  const players = [
    ...game.team_picks
      .filter((p) => p.side === side)
      .map((p) => ({
        id: p.user_id,
        nickname: p.nickname,
        champion: p.champion,
        drawn: p.timeout,
        status: null,
      })),
    ...game.hidden_picks
      .filter((p) => p.side === side)
      .map((p) => ({
        id: p.user_id,
        nickname: p.nickname,
        champion: null,
        drawn: false,
        status: "hidden" as const,
      })),
    ...veto.missing
      .filter((p) => p.side === side && p.games.includes(game.game))
      .map((p) => ({
        id: p.user_id,
        nickname: p.nickname,
        champion: null,
        drawn: false,
        status: choosing ? ("choosing" as const) : null,
      })),
  ];
  if (players.length === 0) {
    return <ChampionFigure champion={null} empty={t("toPick")} size={size} />;
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
              <ChampionPortrait
                slug={p.champion.slug}
                name={p.champion.name}
                shape={size === "lg" ? "big" : "roster"}
              />
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
            {p.status && (
              <span
                className={cn(
                  styles.status,
                  p.status === "choosing" && styles.choosing,
                )}
              >
                {t(p.status)}
              </span>
            )}
            {p.champion && p.drawn && (
              <span className={styles.status}>{t("drawn")}</span>
            )}
          </div>
        </li>
      ))}
    </ul>
  );
}
