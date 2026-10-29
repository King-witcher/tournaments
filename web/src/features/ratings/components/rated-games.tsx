import { useLocale, useTranslations } from "next-intl";

import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { DataTable } from "@/shared/ui/data-table";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { Section } from "@/shared/ui/section";
import { StatusTag } from "@/shared/ui/status-tag";
import { formatPoints } from "../lib/format-points";
import styles from "./rated-games.module.scss";

/** Quantos mapas a API manda (o texto diz "até N"). */
const RECENT = 20;

type Props = {
  tenant: string;
  games: components["schemas"]["RatedGame"][];
};

/**
 * "Últimos mapas" na página da pessoa: torneio (a linha abre a partida),
 * adversário, mapa, resultado, pontos e o rating depois, do mais recente ao
 * mais antigo. Só os torneios que quem vê vê (a API filtra).
 */
export function RatedGames({ tenant, games }: Props) {
  const t = useTranslations("Ratings.games");
  const locale = useLocale();

  return (
    <Section
      title={t("title")}
      description={t("description", { count: RECENT })}
    >
      <DataTable
        label={t("label")}
        columns={[
          { key: "tournament", header: t("tournament") },
          { key: "opponent", header: t("opponent") },
          { key: "map", header: t("map"), secondary: true },
          { key: "result", header: t("result"), secondary: true },
          { key: "points", header: t("points") },
          { key: "after", header: t("after"), secondary: true },
          { key: "when", header: t("when"), secondary: true },
        ]}
        rows={games.map((g) => {
          const name = localizedName(g.tournament, locale);
          return {
            id: `${g.tournament.slug}:${g.match}:${g.game}`,
            href: `/${tenant}/${g.tournament.slug}/matches/${g.match}`,
            search: `${name} ${g.opponent.nickname} ${g.map ?? ""}`,
            cells: {
              tournament: name,
              opponent: (
                <span className={styles.opponent}>
                  <Avatar
                    src={g.opponent.avatar_url}
                    name={g.opponent.nickname}
                  />
                  {g.opponent.active
                    ? g.opponent.nickname
                    : `${g.opponent.nickname} (${t("left")})`}
                </span>
              ),
              map: g.map
                ? t("mapValue", { map: g.map, number: g.game })
                : t("noMap", { number: g.game }),
              result: (
                <StatusTag status={g.won ? "confirmed" : "neutral"} size="sm">
                  {g.won ? t("won") : t("lost")}
                </StatusTag>
              ),
              points: (
                <span className={styles.number}>
                  {formatPoints(g.delta, locale)}
                </span>
              ),
              after: <span className={styles.number}>{g.rating_after}</span>,
              when: <LocalDateTime value={g.at} />,
            },
          };
        })}
        empty={{ text: t("empty") }}
      />
    </Section>
  );
}
