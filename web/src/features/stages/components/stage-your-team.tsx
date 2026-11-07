import { useTranslations } from "next-intl";

import { useTeamName } from "@/features/teams/hooks/use-team-name";
import type { components } from "@/shared/api/schema";
import { Stack } from "@/shared/ui/carbon/stack";
import { candidateState } from "../lib/candidates";
import { StageCheckInButton } from "./stage-check-in-button";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  slug: string;
  entry: Schemas["StageEntry"];
  /** O time de quem vê entre os candidatos. */
  candidate: Schemas["StageCandidate"];
  /** No duelo, o texto fala com a pessoa, sem "time". */
  tdm: boolean;
  /** Quem vê também organiza: o check-in dele vem como terciário. */
  organizer: boolean;
};

/**
 * O que o jogador do time candidato faz e lê na fase seguinte: com o
 * check-in aberto, fazer o check-in (ou desfazer); suplente, quando entra.
 * A situação (suplente, desistiu) já está na etiqueta de "Sua situação".
 * Sem nada a dizer ou fazer, nada aparece.
 */
export function StageYourTeam({
  tenant,
  slug,
  entry,
  candidate,
  tdm,
  organizer,
}: Props) {
  const t = useTranslations("Stages.yourTeam");
  const teamName = useTeamName();
  const state = candidateState(candidate, entry);
  if (state === "withdrawn") return null;
  const checkInOpen = entry.check_in && entry.status === "check_in";
  const note =
    state === "substitute" &&
    t("substitute", {
      tdm: tdm ? "yes" : "no",
      checkIn: entry.check_in ? "yes" : "no",
    });
  if (!note && !checkInOpen) return null;

  return (
    <Stack gap={3}>
      {note && <p>{note}</p>}
      {checkInOpen && (
        <StageCheckInButton
          tenant={tenant}
          slug={slug}
          stage={entry.number}
          team={candidate.team.id}
          name={teamName(candidate.team)}
          checkedIn={candidate.checked_in}
          variant="self"
          kind={organizer ? "tertiary" : "primary"}
        />
      )}
    </Stack>
  );
}
