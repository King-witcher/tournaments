import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Section } from "@/shared/ui/section";
import { Crosstable } from "./crosstable";
import { MatchViews } from "./match-views";
import { RoundsList } from "./rounds-list";

type Props = {
  tenant: string;
  /** Slug do torneio. */
  tournament: string;
  bracket: components["schemas"]["TournamentBracket"];
};

/**
 * As séries do round robin e do suíço. No round robin, por rodada ou na tabela
 * cruzada, à escolha; no suíço, só por rodada, porque nem todos se enfrentam.
 */
export function TableMatches({ tenant, tournament, bracket }: Props) {
  const t = useTranslations("Bracket");
  const swiss = bracket.format.format === "swiss";
  const paired = Math.max(0, ...bracket.matches.map((m) => m.round));
  const rounds = (
    <RoundsList tenant={tenant} tournament={tournament} bracket={bracket} />
  );

  return (
    <Section
      title={t("matchesTitle")}
      description={
        bracket.status === "in_progress" &&
        (swiss
          ? t("swissProgress", {
              round: paired,
              total: bracket.total_rounds ?? 0,
            })
          : t("roundRobinHint"))
      }
    >
      {swiss ? (
        rounds
      ) : (
        <MatchViews
          rounds={rounds}
          crosstable={
            <Crosstable
              tenant={tenant}
              tournament={tournament}
              bracket={bracket}
            />
          }
        />
      )}
    </Section>
  );
}
