import { useLocale, useTranslations } from "next-intl";

import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { soloMember, teamLeft } from "@/features/teams/lib/team-name";
import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { Avatar } from "@/shared/ui/avatar";
import { DataTable } from "@/shared/ui/data-table";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { Section } from "@/shared/ui/section";
import { StatusTag } from "@/shared/ui/status-tag";
import { formatPoints } from "../lib/format-points";
import styles from "./rated-games.module.scss";

/** Quantas partidas a API manda (o texto diz "até N"). */
const RECENT = 20;

type Props = {
  tenant: string;
  games: components["schemas"]["RatedGame"][];
};

/**
 * "Últimas partidas" na página da pessoa: torneio (a linha abre a série),
 * adversário, mapa, resultado, pontos e o rating depois, da mais recente à
 * mais antiga. Sem nenhuma, quem chama não mostra a seção (o "ainda sem
 * rating" acima já diz).
 */
export function RatedGames({ tenant, games }: Props) {
  const t = useTranslations("Ratings.games");
  const locale = useLocale();
  const teamName = useTeamName();

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
          const opponent = teamName(g.opponent);
          const solo = soloMember(g.opponent);
          return {
            id: `${g.tournament.slug}:${g.match}:${g.game}`,
            href: `/${tenant}/${g.tournament.slug}/matches/${g.match}`,
            search: `${name} ${opponent} ${g.map ?? ""}`,
            cells: {
              tournament: name,
              opponent: (
                <span className={styles.opponent}>
                  <Avatar src={solo?.avatar_url} name={opponent} />
                  {teamLeft(g.opponent)
                    ? `${opponent} (${t("left")})`
                    : opponent}
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
      />
    </Section>
  );
}
