import { useTranslations } from "next-intl";

import { shownNickname } from "@/features/members/lib/nickname";
import type { components } from "@/shared/api/schema";
import styles from "./veto-board.module.scss";

type Schemas = components["schemas"];

type Props = {
  veto: Schemas["VetoView"];
  names: { a: string; b: string };
};

/**
 * O que o veto já decidiu para a série, à vista durante ela: os bans de
 * campeão (valem para todas as partidas) e, partida a partida, o mapa (e
 * quem o escolheu) e o campeão de cada lado; no TDM, o de cada jogador, com
 * o sorteado pelo fim da janela marcado e quem ainda está escolhendo.
 */
export function VetoGames({ veto, names }: Props) {
  const t = useTranslations("Match.veto.games");
  const tMember = useTranslations("Member");
  const nick = (n: string) => shownNickname(n, tMember("deletedAccount"));
  const choosing = (game: number, side: Schemas["Side"]) =>
    veto.missing
      .filter((p) => p.side === side && p.games.includes(game))
      .map((p) => t("choosing", { name: nick(p.nickname) }));
  const games = veto.games.filter(
    (g) =>
      g.map ||
      g.pick_a ||
      g.pick_b ||
      g.team_picks.length > 0 ||
      veto.missing.some((p) => p.games.includes(g.game)),
  );
  if (games.length === 0 && veto.bans.length === 0) return null;

  return (
    <div className={styles.board}>
      <h3 className={styles.heading}>{t("title")}</h3>
      {veto.bans.length > 0 && (
        <p className={styles.summary}>
          {t("bans", {
            list: veto.bans
              .map((b) =>
                b.timeout
                  ? t("drawn", { champion: b.champion.name })
                  : b.champion.name,
              )
              .map((name, i) => `${name} (${names[veto.bans[i].side]})`)
              .join(", "),
          })}
        </p>
      )}
      <ul className={styles.list}>
        {games.map((g) => {
          const parts = [
            g.map
              ? g.picked_by
                ? t("mapPicked", { map: g.map.name, name: names[g.picked_by] })
                : t("map", { map: g.map.name })
              : null,
            ...(["a", "b"] as const).map((s) => {
              const pick = g[`pick_${s}`];
              return pick
                ? t("pick", { name: names[s], champion: pick.name })
                : null;
            }),
            // TDM: o campeão de cada jogador, por time.
            ...(["a", "b"] as const).map((s) => {
              const list = [
                ...g.team_picks
                  .filter((p) => p.side === s)
                  .map((p) =>
                    t("teamPick", {
                      name: nick(p.nickname),
                      champion: p.timeout
                        ? t("drawn", { champion: p.champion.name })
                        : p.champion.name,
                    }),
                  ),
                ...choosing(g.game, s),
              ];
              return list.length > 0
                ? t("teamPicks", { name: names[s], list: list.join(", ") })
                : null;
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
