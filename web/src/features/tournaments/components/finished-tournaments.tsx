import { getLocale, getTranslations } from "next-intl/server";

import type { components } from "@/shared/api/schema";
import { DataTable } from "@/shared/ui/data-table";
import { LocalDate } from "@/shared/ui/local-date";
import { Section } from "@/shared/ui/section";
import { localizedName } from "../lib/localize";

/**
 * Os torneios encerrados da comunidade, em tabela (nome, formato, início,
 * inscritos), na ordem da API (mais novo primeiro): a lista que só cresce,
 * com busca e paginação. Sem nenhum, a seção não aparece.
 */
export async function FinishedTournaments({
  slug,
  tournaments,
}: {
  slug: string;
  tournaments: components["schemas"]["TournamentSummary"][];
}) {
  if (tournaments.length === 0) return null;
  const [t, locale] = await Promise.all([
    getTranslations("Tournaments"),
    getLocale(),
  ]);

  return (
    <Section title={t("finished")}>
      <DataTable
        label={t("finished")}
        columns={[
          { key: "name", header: t("columns.name") },
          { key: "format", header: t("columns.format"), secondary: true },
          { key: "starts", header: t("columns.starts"), secondary: true },
          {
            key: "participants",
            header: t("columns.participants"),
            secondary: true,
          },
        ]}
        rows={tournaments.map((tournament) => {
          const name = localizedName(tournament, locale);
          return {
            id: tournament.slug,
            href: `/${slug}/${tournament.slug}`,
            search: `${name} ${tournament.name}`,
            cells: {
              name,
              // O mesmo texto do `useModeLabel` ("Duelo", "TDM 2v2").
              format:
                tournament.mode === "tdm"
                  ? t("modeTeams", { size: tournament.team_size })
                  : t(`mode.${tournament.mode}`),
              starts: tournament.starts_at ? (
                <LocalDate value={tournament.starts_at} />
              ) : (
                t("noDate")
              ),
              participants: t("participants", {
                count: tournament.participants,
                max: tournament.max_participants,
              }),
            },
          };
        })}
      />
    </Section>
  );
}
