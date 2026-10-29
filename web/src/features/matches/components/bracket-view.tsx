import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { BracketGrid } from "./bracket-grid";
import { BracketTabs } from "./bracket-tabs";

type Schemas = components["schemas"];
type Part = Schemas["BracketPart"];

type Props = {
  tenant: string;
  tournament: string;
  bracket: Schemas["TournamentBracket"];
};

/**
 * As abas da dupla: a chave principal segue até a grande final (quem vence
 * a final dela vai para a direita, na mesma grade); a repescagem fica à
 * parte.
 */
const DOUBLE: Part[][] = [["winners", "grand_final"], ["losers"]];

/**
 * Chave inteira: na simples, uma grade de rodadas; na dupla, uma aba para a
 * chave principal (com a grande final) e outra para a repescagem. Abre na
 * aba da partida que espera quem vê, se ela estiver na repescagem.
 */
export function BracketView({ tenant, tournament, bracket }: Props) {
  const t = useTranslations("Bracket");
  const single = bracket.format.bracket === "single";
  const entrants = new Map(bracket.entrants.map((e) => [e.id, e]));
  const grid = (parts: Part[]) => (
    <BracketGrid
      tenant={tenant}
      parts={parts}
      matches={bracket.matches}
      single={single}
      entrants={entrants}
      basePath={`/${tenant}/${tournament}/matches`}
    />
  );

  if (single) return grid(["winners"]);

  const open = bracket.matches.find(
    (m) => m.yours && (m.state === "ready" || m.state === "pending"),
  );
  return (
    <BracketTabs
      label={t("partsLabel")}
      defaultIndex={Math.max(
        0,
        DOUBLE.findIndex((parts) => open && parts.includes(open.part)),
      )}
      tabs={DOUBLE.map((parts) => ({
        key: parts[0],
        label: t(`parts.${parts[0]}`),
        content: grid(parts),
      }))}
    />
  );
}
