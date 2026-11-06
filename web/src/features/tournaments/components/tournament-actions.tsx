import { NextStepButton } from "@/features/lifecycle/components/next-step-button";
import { ParticipationAction } from "@/features/participants/components/participation-action";
import type { components } from "@/shared/api/schema";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Organizer ou owner. */
  organizer: boolean;
  /** Referência para saber se a abertura do check-in já passou. */
  now: Date;
};

/**
 * As ações do cabeçalho, com o primário à direita: para quem joga,
 * inscrever-se ou fazer o check-in; para a organização, o próximo passo do
 * ciclo, com a inscrição e o check-in dela mesma como terciários. Editar é a
 * aba Configurações.
 */
export function TournamentActions({
  tenant,
  tournament,
  organizer,
  now,
}: Props) {
  return (
    <>
      <ParticipationAction
        tenant={tenant}
        tournament={tournament}
        kind={organizer ? "tertiary" : "primary"}
      />
      {organizer && (
        <NextStepButton
          tenant={tenant}
          tournament={tournament}
          now={now}
          kind="primary"
        />
      )}
    </>
  );
}
