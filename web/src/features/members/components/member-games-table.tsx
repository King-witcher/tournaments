"use client";

import { useLocale, useTranslations } from "next-intl";

import { useMatchLabels } from "@/features/matches/hooks/use-match-labels";
import { formatPoints } from "@/features/ratings/lib/format-points";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { DataTable } from "@/shared/ui/data-table";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { useSearchTransition } from "../hooks/use-search-transition";
import { type GamesQuery, gamesHref } from "../lib/games-href";
import styles from "./member-games-table.module.scss";
import { OpponentName } from "./opponent-name";

type Props = {
  tenant: string;
  /** A página da pessoa. */
  base: string;
  page: components["schemas"]["MemberGames"];
  query: GamesQuery;
  /** Algum filtro está valendo: o vazio diz que é do filtro. */
  filtered: boolean;
};

/**
 * A tabela da aba Partidas: torneio (a linha abre a série), a série pelo
 * nome curto (com a fase da 2 em diante), mapa, adversário,
 * resultado com os frags e, nos mapas que contaram para o rating, os pontos
 * e o rating depois; do mais recente. A página vem da API, e trocar de
 * página muda o endereço (`SearchTransition`).
 */
export function MemberGamesTable({
  tenant,
  base,
  page,
  query,
  filtered,
}: Props) {
  const t = useTranslations("Member.games");
  const locale = useLocale();
  const teamName = useTeamName();
  // O nome da série depende de a fase ser eliminação simples.
  const singleLabels = useMatchLabels(true);
  const labels = useMatchLabels(false);
  const { navigate } = useSearchTransition();

  return (
    <DataTable
      label={t("label")}
      columns={[
        { key: "tournament", header: t("tournament") },
        { key: "series", header: t("series"), secondary: true },
        { key: "map", header: t("map"), secondary: true },
        { key: "opponent", header: t("opponent") },
        { key: "result", header: t("result") },
        { key: "points", header: t("points"), secondary: true },
        { key: "after", header: t("after"), secondary: true },
        { key: "when", header: t("when"), secondary: true },
      ]}
      rows={page.games.map((g) => {
        const result = g.won ? t("won") : t("lost");
        return {
          id: `${g.tournament.slug}:${g.match}:${g.game}`,
          href: `/${tenant}/${g.tournament.slug}/matches/${g.match}`,
          cells: {
            tournament: localizedName(g.tournament, locale),
            series: (g.single ? singleLabels : labels).code(g.match),
            map: g.map
              ? t("mapValue", { map: g.map.name, number: g.game })
              : t("noMap", { number: g.game }),
            opponent: (
              <OpponentName team={g.opponent} name={teamName(g.opponent)} />
            ),
            result:
              g.frags == null || g.opponent_frags == null
                ? result
                : t("score", {
                    result,
                    for: g.frags,
                    against: g.opponent_frags,
                  }),
            points: (
              <span className={styles.number}>
                {g.rating
                  ? formatPoints(g.rating.delta, locale)
                  : t("notRated")}
              </span>
            ),
            after: (
              <span className={styles.number}>
                {g.rating ? g.rating.rating_after : t("notRated")}
              </span>
            ),
            when: <LocalDateTime value={g.at} />,
          },
        };
      })}
      paging={{
        page: page.page,
        pageSize: page.page_size,
        totalItems: page.total,
        onPageChange: (next) =>
          navigate(gamesHref(base, { ...query, page: next })),
      }}
      empty={{ text: filtered ? t("emptyFiltered") : t("empty") }}
    />
  );
}
