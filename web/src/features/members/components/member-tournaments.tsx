import { useLocale, useTranslations } from "next-intl";

import { localizedName } from "@/features/tournaments/lib/localize";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { DataTable } from "@/shared/ui/data-table";
import { Section } from "@/shared/ui/section";
import { StatusTag } from "@/shared/ui/status-tag";
import { SITUATION_STATUS } from "../lib/situation";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  profile: Schemas["MemberProfile"];
  /** A página é de quem vê: o vazio fala com ele. */
  isYou: boolean;
};

/**
 * "Torneios nesta comunidade" na página da pessoa: torneio (link), estado,
 * a situação dela (as cores de "Seus torneios") e a colocação, do início
 * mais recente ao mais antigo (a ordem da API).
 */
export function MemberTournaments({ tenant, profile, isYou }: Props) {
  const t = useTranslations("Member.tournaments");
  const tStatus = useTranslations("TournamentStatus");
  const locale = useLocale();
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
          ({ tournament, situation, placement }) => {
            const name = localizedName(tournament, locale);
            const placed = placement ? place(placement) : undefined;
            return {
              id: tournament.slug,
              href: `/${tenant}/${tournament.slug}`,
              search: `${name} ${tournament.name}`,
              cells: {
                tournament: name,
                status: (
                  <StatusTag status={tournament.status}>
                    {tStatus(tournament.status)}
                  </StatusTag>
                ),
                situation: (
                  <StatusTag status={SITUATION_STATUS[situation]}>
                    {t(`situations.${situation}`, {
                      place: placed ?? t("noPlacement"),
                    })}
                  </StatusTag>
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
