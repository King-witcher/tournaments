import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import styles from "./veto-board.module.scss";

type Schemas = components["schemas"];

type Props = {
  veto: Schemas["VetoView"];
  names: { a: string; b: string };
};

/**
 * O que o veto já decidiu, jogo a jogo: o mapa (e quem o escolheu), os
 * campeões banidos e o de cada lado.
 */
export function VetoGames({ veto, names }: Props) {
  const t = useTranslations("Match.veto.games");
  const games = veto.games.filter(
    (g) =>
      g.map ||
      g.pick_a ||
      g.pick_b ||
      g.bans.length > 0 ||
      g.team_picks.length > 0,
  );
  if (games.length === 0) return null;

  return (
    <div className={styles.board}>
      <h3 className={styles.heading}>{t("title")}</h3>
      <ul className={styles.list}>
        {games.map((g) => {
          const parts = [
            g.map
              ? g.picked_by
                ? t("mapPicked", { map: g.map.name, name: names[g.picked_by] })
                : t("map", { map: g.map.name })
              : null,
            g.bans.length > 0
              ? t("bans", {
                  list: g.bans
                    .map((b) => `${b.champion.name} (${names[b.side]})`)
                    .join(", "),
                })
              : null,
            ...(["a", "b"] as const).map((s) => {
              const pick = g[`pick_${s}`];
              return pick
                ? t("pick", { name: names[s], champion: pick.name })
                : null;
            }),
            // TDM: o campeão de cada jogador, por time.
            ...(["a", "b"] as const).map((s) => {
              const mine = g.team_picks.filter((p) => p.side === s);
              if (mine.length === 0) return null;
              const list = mine
                .map((p) => `${p.nickname} (${p.champion.name})`)
                .join(", ");
              return t("teamPicks", { name: names[s], list });
            }),
          ].filter(Boolean);
          return (
            <li key={g.game} className={styles.item}>
              <span className={styles.name}>{t("game", { game: g.game })}</span>
              <span className={styles.summary}>{parts.join(" · ")}</span>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
