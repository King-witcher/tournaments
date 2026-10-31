import { useTranslations } from "next-intl";
import type { ReactNode } from "react";
import { TournamentProgress } from "@/features/tournaments/components/tournament-progress";
import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";
import { Section } from "@/shared/ui/section";
import styles from "./lifecycle-section.module.scss";
import { NextStepButton } from "./next-step-button";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
  /** Referência para saber se a abertura do check-in já passou. */
  now: Date;
};

/**
 * Na aba Organização, o quadro do estado: o ciclo com o passo de agora, o
 * que vale neste passo (quando o check-in abre,
 * quem entra na chave) e o próximo passo como botão, com o motivo embaixo
 * quando ainda não dá (no rascunho, a data de início que ficou para trás).
 */
export function LifecycleSection({ tenant, tournament, now }: Props) {
  const t = useTranslations("Lifecycle");
  const tTournament = useTranslations("Tournament");
  const { status } = tournament;
  const hasCheckIn = tournament.check_in_minutes != null;
  const opensAt = tournament.check_in_opens_at;
  // No TDM, quem entra na chave são os times, não os inscritos.
  const teams = tournament.mode === "tdm";

  let hint: ReactNode = null;
  if (status === "draft") {
    hint = tTournament("draftHint");
  } else if (status === "registration" && hasCheckIn) {
    hint =
      opensAt &&
      new Date(opensAt) > now &&
      t.rich("opensAt", { date: () => <LocalDateTime value={opensAt} /> });
  } else if (status === "registration") {
    hint = t(teams ? "noCheckInTdm" : "noCheckIn");
  } else if (status === "check_in") {
    hint = t(teams ? "checkInOpenTdm" : "checkInOpen");
  } else if (status === "finished") {
    hint = t("finished");
  }

  return (
    <Section title={t("title")}>
      <TournamentProgress tournament={tournament} />
      {hint && <p className={styles.hint}>{hint}</p>}
      <NextStepButton
        tenant={tenant}
        tournament={tournament}
        now={now}
        kind="secondary"
      />
    </Section>
  );
}
