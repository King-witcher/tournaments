import { useTranslations } from "next-intl";

import { PublishTournamentButton } from "@/features/tournaments/components/publish-tournament-button";
import type { components } from "@/shared/api/schema";
import { StartButton } from "./start-button";
import { TransitionButton } from "./transition-button";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * O próximo passo do ciclo, para organizer e owner: publicar (rascunho),
 * abrir o check-in (inscrição com check-in) ou iniciar (inscrição sem
 * check-in, ou no check-in). Em andamento ou encerrado, nada. Voltar atrás
 * fica no ⋯ (voltar para rascunho, fechar o check-in, desfazer o início) e a
 * correção de resultados, na partida.
 */
export function NextStepButton({ tenant, tournament }: Props) {
  const t = useTranslations("Lifecycle");
  const { status, slug } = tournament;

  if (status === "draft") {
    return <PublishTournamentButton tenant={tenant} tournament={tournament} />;
  }
  if (status === "registration" && tournament.check_in) {
    return (
      <TransitionButton
        tenant={tenant}
        slug={slug}
        transition="open_check_in"
        label={t("openCheckIn")}
      />
    );
  }
  if (status === "registration" || status === "check_in") {
    return <StartButton tenant={tenant} tournament={tournament} />;
  }
  return null;
}
