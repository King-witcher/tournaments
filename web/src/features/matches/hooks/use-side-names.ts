import { useTeamName } from "@/features/teams/hooks/use-team-name";
import type { components } from "@/shared/api/schema";

/** O nome exibido de cada lado da partida (vazio no lado ainda sem time). */
export function useSideNames(detail: components["schemas"]["MatchDetail"]) {
  const teamName = useTeamName();
  return {
    a: detail.a ? teamName(detail.a) : "",
    b: detail.b ? teamName(detail.b) : "",
  };
}
