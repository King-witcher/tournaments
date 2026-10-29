import { getTranslations } from "next-intl/server";

import type { components } from "@/shared/api/schema";
import { Link } from "@/shared/ui/link";
import { Notice } from "@/shared/ui/notice";
import { checkInShown, formationCounts } from "../lib/formation";
import { getFormation } from "../server";

type Props = {
  tenant: string;
  tournament: components["schemas"]["Tournament"];
};

/**
 * Na ordem da chave do TDM, o aviso de que há times fora dela (incompletos
 * ou com alguém sem check-in), com o link para a formação: só os completos
 * entram na ordem, e o torneio não começa com time incompleto. Sem time
 * incompleto (ou sem a formação), nada.
 */
export async function SeedingTeamsNotice({ tenant, tournament }: Props) {
  const [t, tTeam, formation] = await Promise.all([
    getTranslations("Participants.seeding"),
    getTranslations("Team"),
    getFormation(tenant, tournament.slug),
  ]);
  if (!formation) return null;
  const checkIn = checkInShown(tournament.status, formation);
  const { incomplete } = formationCounts(formation, checkIn);
  if (incomplete === 0) return null;
  return (
    <Notice
      kind="warning"
      action={
        <Link href={`/${tenant}/${tournament.slug}/teams`}>
          {tTeam("form")}
        </Link>
      }
    >
      {t(checkIn ? "teamsLeftOutCheckIn" : "teamsLeftOut", {
        count: incomplete,
      })}
    </Notice>
  );
}
