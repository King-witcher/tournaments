import { useTranslations } from "next-intl";

import { useTeamName } from "@/features/teams/hooks/use-team-name";
import type { components } from "@/shared/api/schema";
import { DataTable } from "@/shared/ui/data-table";
import { StatusTag } from "@/shared/ui/status-tag";
import { type CandidateState, candidateState } from "../lib/candidates";
import { StageCheckInButton } from "./stage-check-in-button";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  slug: string;
  entry: Schemas["StageEntry"];
  /** Organizer ou owner: marca ou desmarca o check-in de qualquer time. */
  organizer: boolean;
  /** No duelo, a coluna é "Jogador". */
  tdm: boolean;
};

const STATE_STATUS = {
  enters: "confirmed",
  substitute: "neutral",
  absent: "check_in",
  withdrawn: "cancelled",
} as const satisfies Record<CandidateState, string>;

/**
 * Os candidatos à fase seguinte, na ordem da tabela da fase anterior: a ordem
 * com que entram (os suplentes e quem desistiu, sem), a posição na fase
 * anterior e, na fase com check-in, se fez. Com o check-in aberto, a
 * organização marca ou desmarca o de qualquer time. No duelo, a coluna do
 * time é "Jogador".
 */
export function StageCandidates({
  tenant,
  slug,
  entry,
  organizer,
  tdm,
}: Props) {
  const t = useTranslations("Stages.list");
  const teamName = useTeamName();
  const checkInOpen = entry.check_in && entry.status === "check_in";

  const rows = entry.candidates.map((candidate) => {
    const state = candidateState(candidate, entry);
    const name = teamName(candidate.team);
    return {
      id: String(candidate.team.id),
      search: name,
      cells: {
        seed: candidate.projected_seed ?? "—",
        team: name,
        previous: t("rank", { rank: candidate.previous_rank }),
        presence: (
          <StatusTag status={STATE_STATUS[state]} size="sm">
            {t(`state.${state}`)}
          </StatusTag>
        ),
      },
      actions:
        organizer && checkInOpen && state !== "withdrawn" ? (
          <StageCheckInButton
            tenant={tenant}
            slug={slug}
            stage={entry.number}
            team={candidate.team.id}
            name={name}
            checkedIn={candidate.checked_in}
            variant="row"
          />
        ) : undefined,
    };
  });

  return (
    <DataTable
      label={t("title", { number: entry.number })}
      primaryColumn="team"
      columns={[
        // No celular, sem a ordem: as linhas já estão nela (quem entra segue
        // a tabela da fase anterior), e o ⋯ da organização cabe.
        { key: "seed", header: t("seed"), secondary: true },
        { key: "team", header: t(tdm ? "team" : "player") },
        {
          key: "previous",
          header: t("previous", { previous: entry.number - 1 }),
          secondary: true,
        },
        { key: "presence", header: t("presence") },
      ]}
      rows={rows}
      empty={{ text: t("empty") }}
    />
  );
}
