import { useTranslations } from "next-intl";

import { NextStepReason } from "@/features/lifecycle/components/next-step-reason";
import { useLifecycleHint } from "@/features/lifecycle/hooks/use-lifecycle-hint";
import type { components } from "@/shared/api/schema";
import { Section } from "@/shared/ui/section";
import { useProgressNotes } from "../hooks/use-progress-notes";
import { TournamentProgress } from "./tournament-progress";
import styles from "./tournament-state-section.module.scss";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  tournament: Schemas["Tournament"];
  /** A chave da fase atual, a partir do início: o andamento e o campeão. */
  bracket?: Schemas["TournamentBracket"];
  /** Organizer ou owner: vê o que vale no passo de agora. */
  organizer: boolean;
  now: Date;
};

/**
 * Na Visão geral, onde o torneio está: o ciclo e, embaixo do passo, em
 * andamento, quantas séries da fase atual já se decidiram; encerrado, o
 * campeão (`useProgressNotes`). Para a organização, o que vale no passo de
 * agora e, quando o próximo passo não dá, o motivo; o botão é o do cabeçalho.
 */
export function TournamentStateSection({
  tenant,
  tournament,
  bracket,
  organizer,
  now,
}: Props) {
  const t = useTranslations("Lifecycle");
  const notes = useProgressNotes();
  const hint = useLifecycleHint(tournament, now);
  return (
    <Section title={t("title")}>
      <TournamentProgress
        tournament={tournament}
        secondary={notes(tournament, bracket)}
      />
      {organizer && hint && <p className={styles.hint}>{hint}</p>}
      {organizer && (
        <NextStepReason tenant={tenant} tournament={tournament} now={now} />
      )}
    </Section>
  );
}
