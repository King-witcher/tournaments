import { useLocale, useTranslations } from "next-intl";

import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { DataTable } from "@/shared/ui/data-table";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { Section } from "@/shared/ui/section";
import { OpponentName } from "./opponent-name";

type Props = {
  tenant: string;
  /** A página da pessoa, para "Ver todas as partidas". */
  base: string;
  series: components["schemas"]["MemberSeries"][];
};

/**
 * "Últimas séries" na Visão geral da pessoa: torneio (a linha abre a série),
 * adversário, resultado com o placar em mapas (ou o W.O.) e quando, da mais
 * recente; "Ver todas as partidas" leva à aba Partidas. Sem nenhuma, quem
 * chama não mostra a seção.
 */
export function RecentSeries({ tenant, base, series }: Props) {
  const t = useTranslations("Member.recent");
  const locale = useLocale();
  const teamName = useTeamName();

  return (
    <Section
      title={t("title")}
      action={
        <ButtonLink href={`${base}/games`} kind="ghost" size="sm">
          {t("all")}
        </ButtonLink>
      }
    >
      <DataTable
        label={t("label")}
        searchable={false}
        columns={[
          { key: "tournament", header: t("tournament") },
          { key: "opponent", header: t("opponent") },
          { key: "result", header: t("result") },
          { key: "when", header: t("when"), secondary: true },
        ]}
        rows={series.map((s) => {
          const outcome = t(s.outcome);
          return {
            id: `${s.tournament.slug}:${s.match}`,
            href: `/${tenant}/${s.tournament.slug}/matches/${s.match}`,
            cells: {
              tournament: localizedName(s.tournament, locale),
              opponent: (
                <OpponentName team={s.opponent} name={teamName(s.opponent)} />
              ),
              result:
                s.wins == null || s.losses == null
                  ? t("walkover", { outcome })
                  : t("score", { outcome, wins: s.wins, losses: s.losses }),
              when: <LocalDateTime value={s.at} />,
            },
          };
        })}
      />
    </Section>
  );
}
