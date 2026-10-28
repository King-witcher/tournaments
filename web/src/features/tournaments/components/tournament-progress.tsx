import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { ProgressSteps } from "@/shared/ui/progress-steps";

type Status = components["schemas"]["TournamentStatus"];

const STEPS: Status[] = [
  "draft",
  "registration",
  "check_in",
  "in_progress",
  "finished",
];

type Props = {
  tournament: Pick<
    components["schemas"]["Tournament"],
    "status" | "check_in_minutes"
  >;
};

/**
 * O ciclo do torneio com o passo de agora, só leitura: rascunho → inscrições
 * → check-in → em andamento → encerrado. Sem check-in, o passo não existe
 * (a inscrição vai direto para em andamento); encerrado, todos ficam feitos.
 */
export function TournamentProgress({ tournament }: Props) {
  const t = useTranslations("Tournament");
  const tStatus = useTranslations("TournamentStatus");
  const steps = STEPS.filter(
    (step) =>
      step !== "check_in" ||
      tournament.check_in_minutes != null ||
      tournament.status === "check_in",
  );
  const current =
    tournament.status === "finished"
      ? steps.length
      : steps.indexOf(tournament.status);

  return (
    <ProgressSteps
      label={t("progress")}
      current={current}
      steps={steps.map((step) => ({ key: step, label: tStatus(step) }))}
    />
  );
}
