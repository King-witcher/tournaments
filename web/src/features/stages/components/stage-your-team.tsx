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
};

/**
 * O que o jogador do time candidato lê na fase seguinte: suplente, quando
 * entra; com o check-in feito e ainda aberto, desfazer. Fazer o check-in é o
 * aviso acima do bloco (`StageCheckInNotice`). A situação (suplente,
 * desistiu) já está na etiqueta de "Sua situação".
 * Sem nada a dizer ou fazer, nada aparece.
 */
export function StageYourTeam({ tenant, slug, entry, candidate, tdm }: Props) {
  const t = useTranslations("Stages.yourTeam");
  const teamName = useTeamName();
  const state = candidateState(candidate, entry);
  if (state === "withdrawn") return null;
  const canUndo =
    entry.check_in && entry.status === "check_in" && candidate.checked_in;
  const note =
    state === "substitute" &&
    t("substitute", {
      tdm: tdm ? "yes" : "no",
      checkIn: entry.check_in ? "yes" : "no",
    });
  if (!note && !canUndo) return null;

  return (
    <Stack gap={3}>
      {note && <p>{note}</p>}
      {canUndo && (
        <StageCheckInButton
          tenant={tenant}
          slug={slug}
          stage={entry.number}
          team={candidate.team.id}
          name={teamName(candidate.team)}
          checkedIn
          variant="self"
        />
      )}
    </Stack>
  );
}
