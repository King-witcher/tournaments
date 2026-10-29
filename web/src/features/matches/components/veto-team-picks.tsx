import { useTranslations } from "next-intl";

import { shownNickname } from "@/features/members/lib/nickname";
import type { components } from "@/shared/api/schema";
import styles from "./veto-board.module.scss";

type Schemas = components["schemas"];

type Props = {
  /** O jogo em andamento. */
  game: Schemas["VetoGame"];
  veto: Schemas["VetoView"];
  names: { a: string; b: string };
};

/**
 * TDM: a escolha simultânea de campeões do mapa em andamento, ao vivo: por
 * time, o campeão de cada jogador (com o sorteado pelo fim da janela
 * marcado) e quem ainda está escolhendo.
 */
export function VetoTeamPicks({ game, veto, names }: Props) {
  const t = useTranslations("Match.veto.teamPicks");
  const tMember = useTranslations("Member");
  const missing = veto.game === game.game ? veto.missing : [];
  if (game.team_picks.length === 0 && missing.length === 0) return null;
  const nick = (n: string) => shownNickname(n, tMember("deletedAccount"));

  return (
    <div className={styles.board}>
      <h3 className={styles.heading}>{t("title", { game: game.game })}</h3>
      <ul className={styles.list}>
        {(["a", "b"] as const).map((side) => {
          const picked = game.team_picks
            .filter((p) => p.side === side)
            .map((p) =>
              p.timeout
                ? `${nick(p.nickname)}: ${t("drawn", { champion: p.champion.name })}`
                : `${nick(p.nickname)}: ${p.champion.name}`,
            );
          const choosing = missing
            .filter((p) => p.side === side)
            .map((p) => `${nick(p.nickname)}: ${t("choosing")}`);
          return (
            <li key={side} className={styles.item}>
              <span className={styles.name}>{names[side]}</span>
              <span className={styles.summary}>
                {[...picked, ...choosing].join(" · ")}
              </span>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
