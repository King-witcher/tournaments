import { useTranslations } from "next-intl";
import { TournamentProgress } from "@/features/tournaments/components/tournament-progress";
import { useProgressNotes } from "@/features/tournaments/hooks/use-progress-notes";
import type { components } from "@/shared/api/schema";
import { Panel } from "@/shared/ui/panel";
import { Section } from "@/shared/ui/section";
import { useLifecycleHint } from "../hooks/use-lifecycle-hint";
import styles from "./lifecycle-section.module.scss";
import { NextStepButton } from "./next-step-button";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Referência para saber se a abertura do check-in já passou. */
  now: Date;
  /** A chave da fase atual, a partir do início: o andamento embaixo do passo. */
  bracket?: components["schemas"]["TournamentBracket"];
};

/**
 * Na Organização, o quadro do estado: o ciclo com o passo de agora (e o
 * andamento das séries, `useProgressNotes`), o que vale neste passo
 * (`useLifecycleHint`) e o próximo passo como botão, com o motivo embaixo
 * quando ainda não dá (no rascunho, a data de início que ficou para trás).
 */
export function LifecycleSection({ tenant, tournament, now, bracket }: Props) {
  const t = useTranslations("Lifecycle");
  const notes = useProgressNotes();
  const hint = useLifecycleHint(tournament, now);
  return (
    <Panel>
      <Section title={t("title")}>
        <TournamentProgress
          tournament={tournament}
          secondary={notes(tournament, bracket)}
        />
        {hint && <p className={styles.hint}>{hint}</p>}
        <NextStepButton
          tenant={tenant}
          tournament={tournament}
          now={now}
          kind="secondary"
        />
      </Section>
    </Panel>
  );
}
