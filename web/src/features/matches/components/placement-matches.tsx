import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Section } from "@/shared/ui/section";
import { RoundsList } from "./rounds-list";

type Props = {
  tenant: string;
  /** Slug do torneio. */
  tournament: string;
  bracket: components["schemas"]["TournamentBracket"];
};

/**
 * As séries da disputa de posições: os pares, todos abertos desde o início,
 * com os mesmos cartões da chave.
 */
export function PlacementMatches({ tenant, tournament, bracket }: Props) {
  const t = useTranslations("Bracket");

  return (
    <Section
      title={t("matchesTitle")}
      description={bracket.status === "in_progress" && t("hint")}
    >
      <RoundsList tenant={tenant} tournament={tournament} bracket={bracket} />
    </Section>
  );
}
