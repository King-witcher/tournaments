import { useTranslations } from "next-intl";

import { PublishTournamentButton } from "@/features/tournaments/components/publish-tournament-button";
import type { components } from "@/shared/api/schema";
import type { ButtonKind } from "@/shared/ui/button";
import { StartButton } from "./start-button";
import { TransitionButton } from "./transition-button";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Referência para saber se a abertura do check-in já passou. */
  now: Date;
  /** `primary` no cabeçalho; `secondary` na aba Organização, embaixo dele. */
  kind: ButtonKind;
};

/**
 * O próximo passo do ciclo, para organizer e owner: publicar (rascunho),
 * abrir o check-in (inscrição com check-in) ou iniciar (inscrição sem
 * check-in, ou no check-in). Em andamento ou encerrado, nada: desfazer o
 * início fica no ⋯ e a correção de resultados, na partida.
 */
export function NextStepButton({ tenant, tournament, now, kind }: Props) {
  const t = useTranslations("Lifecycle");
  const tTournament = useTranslations("Tournament");
  const { status, slug } = tournament;

  if (status === "draft") {
    // Pela data e pela janela, o check-in já abriu: publicado agora, o
    // torneio vai direto para o check-in, sem inscrições.
    const opensAt = tournament.check_in_opens_at;
    return (
      <PublishTournamentButton
        tenant={tenant}
        slug={slug}
        kind={kind}
        skipsRegistration={opensAt != null && new Date(opensAt) <= now}
        disabledReason={
          tournament.starts_at ? undefined : tTournament("publishNeedsDate")
        }
      />
    );
  }
  if (status === "registration" && tournament.check_in_minutes != null) {
    return (
      <TransitionButton
        tenant={tenant}
        slug={slug}
        transition="open_check_in"
        label={t("openCheckIn")}
        kind={kind}
      />
    );
  }
  if (status === "registration" || status === "check_in") {
    return <StartButton tenant={tenant} tournament={tournament} kind={kind} />;
  }
  return null;
}
