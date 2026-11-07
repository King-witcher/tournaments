import { useTranslations } from "next-intl";

import { useTeamName } from "@/features/teams/hooks/use-team-name";
import type { components } from "@/shared/api/schema";
import { useMatchLabels } from "./use-match-labels";

/**
 * A próxima série de quem vê em palavras: o código e o adversário, ou de onde
 * ele vem ("Próxima série: 2.1 contra quem vencer 1.3").
 */
export function useNextMatchLabel(next: components["schemas"]["NextMatch"]) {
  const t = useTranslations("Home.tournaments.next");
  const { code, shortCode } = useMatchLabels(next.single);
  const teamName = useTeamName();
  const match = code(next.key);
  const from = next.opponent_from;
  if (next.opponent) {
    return t("against", { match, opponent: teamName(next.opponent) });
  }
  return from
    ? t(from.outcome, { match, source: shortCode(from.match_key) })
    : t("unknown", { match });
}
