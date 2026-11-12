import { useLocale, useTranslations } from "next-intl";
import { SituationMark } from "@/features/participants/components/situation-mark";
import { useTeamName } from "@/features/teams/hooks/use-team-name";
import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { DataTable } from "@/shared/ui/data-table";
import { Section } from "@/shared/ui/section";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  profile: Schemas["MemberProfile"];
  /** A página é de quem vê: o vazio fala com ele. */
  isYou: boolean;
};

/**
 * "Torneios nesta comunidade" na página da pessoa: torneio (link), estado,
 * a situação dela (texto; a exceção com a marca de "Seus torneios",
 * `SituationMark`; no TDM, com o time) e a colocação, do início
 * mais recente ao mais antigo (a ordem da API).
 */
export function MemberTournaments({ tenant, profile, isYou }: Props) {
  const t = useTranslations("Member.tournaments");
  const tStatus = useTranslations("TournamentStatus");
  const locale = useLocale();
  const teamName = useTeamName();
  const place = (placement: Schemas["MyPlacement"]) =>
    placement.tied > 1
      ? t("range", {
          from: placement.rank,
          to: placement.rank + placement.tied - 1,
        })
      : t("rank", { rank: placement.rank });

  return (
    <Section title={t("title")}>
      <DataTable
        label={t("title")}
        columns={[
          { key: "tournament", header: t("tournament") },
          { key: "status", header: t("status"), secondary: true },
          { key: "situation", header: t("situation") },
          { key: "placement", header: t("placement"), secondary: true },
        ]}
        rows={profile.tournaments.map(
          ({ tournament, situation, placement, team }) => {
            const name = localizedName(tournament, locale);
            // TDM: o time do membro no torneio.
            const label = team
              ? t("withTeam", { tournament: name, team: teamName(team) })
              : name;
            // No empate a decidir, a melhor posição do grupo ("Empate no 3º").
            const placed = placement
              ? situation === "tie_pending"
                ? t("rank", { rank: placement.rank })
                : place(placement)
              : undefined;
            return {
              id: tournament.slug,
              href: `/${tenant}/${tournament.slug}`,
              search: `${name} ${tournament.name}`,
              cells: {
                tournament: label,
                status: tStatus(tournament.status),
                situation: (
                  <SituationMark
                    situation={situation}
                    label={t(`situations.${situation}`, {
                      place: placed ?? t("noPlacement"),
                    })}
                  />
                ),
                placement: placed ?? t("noPlacement"),
              },
            };
          },
        )}
        empty={{
          text: isYou ? t("emptyYou") : t("empty", { name: profile.nickname }),
          action: (
            <ButtonLink href={`/${tenant}`} kind="tertiary">
              {t("browse")}
            </ButtonLink>
          ),
        }}
      />
    </Section>
  );
}
