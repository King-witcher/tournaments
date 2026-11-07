import { useTranslations } from "next-intl";

import { useTeamName } from "@/features/teams/hooks/use-team-name";
import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];

/**
 * A linha curta embaixo dos passos do ciclo do torneio: em andamento,
 * quantas séries da fase atual já se decidiram; encerrado, o campeão. Sem a
 * chave (antes do início, ou a API falhou), nenhuma.
 */
export function useProgressNotes() {
  const t = useTranslations("Tournament.overview");
  const teamName = useTeamName();
  return (
    tournament: Schemas["Tournament"],
    bracket: Schemas["TournamentBracket"] | undefined,
  ): Partial<Record<Schemas["TournamentStatus"], string>> => {
    if (!bracket) return {};
    if (tournament.status === "in_progress") {
      // Bye e o reset que não precisou não se jogam.
      const played = bracket.matches.filter(
        (m) => m.state !== "bye" && m.state !== "not_needed",
      );
      const counts = {
        done: played.filter((m) => m.state === "completed").length,
        total: played.length,
      };
      return {
        in_progress:
          tournament.stages.length > 1
            ? t("decidedStage", { stage: bracket.stage, ...counts })
            : t("decided", counts),
      };
    }
    if (tournament.status === "finished") {
      const first = bracket.standings_final.find(
        (s) => s.rank === 1 && s.tied === 1,
      );
      const team = [...bracket.entrants, ...bracket.other_teams].find(
        (e) => e.id === first?.team_id,
      );
      return team ? { finished: t("champion", { name: teamName(team) }) } : {};
    }
    return {};
  };
}
