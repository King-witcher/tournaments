import { useTranslations } from "next-intl";

import { useTeamName } from "@/features/teams/hooks/use-team-name";
import type { components } from "@/shared/api/schema";
import { Stack } from "@/shared/ui/carbon/stack";
import { Section } from "@/shared/ui/section";
import { PendingTieGroup } from "./pending-tie-group";
import { SeedAllTies } from "./seed-all-ties";

type Props = {
  tenant: string;
  slug: string;
  bracket: components["schemas"]["TournamentBracket"];
};

/**
 * Para a organização, na aba Classificação: os empates do fim da fase que
 * esperam a decisão dela, um grupo por vez, e, com mais de um, a ordem da chave
 * em todos de uma vez. Nada pendente, nada aparece.
 */
export function PendingTies({ tenant, slug, bracket }: Props) {
  const t = useTranslations("Bracket.ties");
  const teamName = useTeamName();
  const ties = bracket.pending_ties;
  if (ties.length === 0) return null;
  const byId = new Map(bracket.entrants.map((e) => [e.id, e]));
  const nameOf = (id: number) => {
    const team = byId.get(id);
    return team ? teamName(team) : "";
  };

  return (
    <Section title={t("title")} description={t("description")}>
      <Stack gap={7}>
        {ties.map((tie) => (
          <Section
            key={tie.team_ids.join("-")}
            title={t("group", { position: tie.position })}
            level={3}
          >
            <PendingTieGroup
              tenant={tenant}
              slug={slug}
              position={tie.position}
              teams={tie.team_ids.map((id) => ({ id, name: nameOf(id) }))}
              last={ties.length === 1}
            />
          </Section>
        ))}
        {ties.length > 1 && <SeedAllTies tenant={tenant} slug={slug} />}
      </Stack>
    </Section>
  );
}
