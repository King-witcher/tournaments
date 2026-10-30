import { registrationOpen } from "@/features/tournaments/lib/status";
import type { components } from "@/shared/api/schema";
import type { ButtonKind } from "@/shared/ui/button";
import { CheckInButton } from "./check-in-button";
import { RegistrationButton } from "./registration-button";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  kind: ButtonKind;
};

/**
 * O que quem vê faz agora com a própria participação, no cabeçalho:
 * inscrever-se (inscrição aberta até o início, sem ser só por link, com vaga)
 * ou fazer o check-in (check-in aberto, inscrito, ainda não fez). O motivo de
 * não poder (só por link, lotado) fica em "Sua situação".
 */
export function ParticipationAction({ tenant, tournament, kind }: Props) {
  const { status, registered, slug } = tournament;
  if (
    registrationOpen(status) &&
    !registered &&
    tournament.admission === "open" &&
    tournament.participants < tournament.max_participants
  ) {
    return <RegistrationButton tenant={tenant} slug={slug} kind={kind} />;
  }
  if (status === "check_in" && registered && !tournament.checked_in) {
    return <CheckInButton tenant={tenant} slug={slug} kind={kind} />;
  }
  return null;
}
