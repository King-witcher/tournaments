import { useTranslations } from "next-intl";

import { NextStepButton } from "@/features/lifecycle/components/next-step-button";
import { ParticipationAction } from "@/features/participants/components/participation-action";
import type { components } from "@/shared/api/schema";
import { ButtonLink } from "@/shared/ui/button-link";
import { textsEditable } from "../lib/status";

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
 * inscrever-se ou fazer o check-in; para a organização, "Editar" (some
 * quando encerra: nada mais muda) e o próximo passo do ciclo, com a
 * inscrição e o check-in dela mesma como terciários.
 */
export function TournamentActions({
  tenant,
  tournament,
  organizer,
  now,
}: Props) {
  const t = useTranslations("Tournament");
  return (
    <>
      {organizer && textsEditable(tournament.status) && (
        <ButtonLink
          href={`/${tenant}/${tournament.slug}/edit`}
          kind="secondary"
        >
          {t("edit")}
        </ButtonLink>
      )}
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
