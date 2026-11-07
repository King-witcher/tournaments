import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import { LocalDateTime } from "@/shared/ui/local-date-time";

/**
 * O que vale no passo de agora, para a organização: no rascunho, que só ela
 * o vê; nas inscrições, quando o check-in abre ou quem entra na chave sem
 * ele; no check-in, quem entra; no fim, onde corrigir. Nada a dizer, `null`.
 */
export function useLifecycleHint(
  tournament: components["schemas"]["Tournament"],
  /** Referência para saber se a abertura do check-in já passou. */
  now: Date,
): ReactNode {
  const t = useTranslations("Lifecycle");
  const tTournament = useTranslations("Tournament");
  const { status } = tournament;
  const opensAt = tournament.check_in_opens_at;
  // No TDM, quem entra na chave são os times, não os inscritos.
  const teams = tournament.mode === "tdm";

  if (status === "draft") return tTournament("draftHint");
  if (status === "registration" && tournament.check_in_minutes != null) {
    return opensAt && new Date(opensAt) > now
      ? t.rich("opensAt", { date: () => <LocalDateTime value={opensAt} /> })
      : null;
  }
  if (status === "registration") return t(teams ? "noCheckInTdm" : "noCheckIn");
  if (status === "check_in") return t(teams ? "checkInOpenTdm" : "checkInOpen");
  if (status === "finished") return t("finished");
  return null;
}
