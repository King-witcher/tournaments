import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";

/**
 * O que vale no passo de agora, para a organização: no rascunho, que só ela
 * o vê; nas inscrições sem check-in, quem entra na chave; no check-in, quem
 * entra; no fim, onde corrigir. Nada a dizer, `null`.
 */
export function useLifecycleHint(
  tournament: components["schemas"]["Tournament"],
): ReactNode {
  const t = useTranslations("Lifecycle");
  const tTournament = useTranslations("Tournament");
  const { status } = tournament;
  // No TDM, quem entra na chave são os times, não os inscritos.
  const teams = tournament.mode === "tdm";

  if (status === "draft") return tTournament("draftHint");
  if (status === "registration" && tournament.check_in) return null;
  if (status === "registration") return t(teams ? "noCheckInTdm" : "noCheckIn");
  if (status === "check_in") return t(teams ? "checkInOpenTdm" : "checkInOpen");
  if (status === "finished") return t("finished");
  return null;
}
