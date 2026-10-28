import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { groupByRound } from "../lib/bracket-layout";
import { BracketPartGrid } from "./bracket-part-grid";

type Schemas = components["schemas"];

type Props = {
  tenant: string;
  tournament: string;
  bracket: Schemas["TournamentBracket"];
};

/**
 * Chave inteira: na simples, uma grade de rodadas; na dupla, a chave de
 * cima, a de baixo e a grande final, cada uma com rolagem horizontal.
 */
export function BracketView({ tenant, tournament, bracket }: Props) {
  const t = useTranslations("Bracket");
  const single = bracket.format.bracket === "single";
  const entrants = new Map(bracket.entrants.map((e) => [e.user_id, e]));

  return (
    <div className="flex flex-col gap-6">
      {groupByRound(bracket.matches).map(({ part, rounds }) => (
        <BracketPartGrid
          key={part}
          title={single ? undefined : t(`parts.${part}`)}
          part={part}
          rounds={rounds}
          single={single}
          entrants={entrants}
          basePath={`/${tenant}/${tournament}/matches`}
        />
      ))}
    </div>
  );
}
