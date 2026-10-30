import { useTranslations } from "next-intl";

import { PublishTournamentButton } from "@/features/tournaments/components/publish-tournament-button";
import { startTiming } from "@/features/tournaments/lib/form-values";
import type { components } from "@/shared/api/schema";
import type { ButtonKind } from "@/shared/ui/button";
import { StartButton } from "./start-button";
import { TransitionButton } from "./transition-button";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Referência para saber se a data de início ficou para trás. */
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
  const { status, slug } = tournament;

  if (status === "draft") {
    // Data que passou ou ficou perto demais desde que o rascunho foi criado:
    // a API recusa, então o botão já vem desabilitado, com o motivo.
    return (
      <PublishTournamentButton
        tenant={tenant}
        slug={slug}
        kind={kind}
        startBehind={
          startTiming(
            tournament.starts_at ?? null,
            tournament.check_in_minutes ?? null,
            now,
          ) ?? undefined
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
