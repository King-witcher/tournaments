import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { teamName } from "../lib/team-name";

/**
 * `teamName` com os textos do idioma de quem vê: devolve a função que dá o
 * nome exibido de um time. Serve a Server Components não assíncronos e a
 * Client Components (o `useTranslations` do next-intl funciona nos dois).
 */
export function useTeamName() {
  const tTeam = useTranslations("Team");
  const tMember = useTranslations("Member");
  return (team: components["schemas"]["Team"]) =>
    teamName(team, {
      deleted: tMember("deletedAccount"),
      teamOf: (captain) => tTeam("teamOf", { captain }),
    });
}
