import { useTranslations } from "next-intl";

import { isSingle } from "@/features/tournaments/lib/format";
import type { components } from "@/shared/api/schema";
import { BracketGrid } from "./bracket-grid";
import { MatchViews } from "./match-views";

type Schemas = components["schemas"];
type Part = Schemas["BracketPart"];

type Props = {
  tenant: string;
  tournament: string;
  bracket: Schemas["TournamentBracket"];
};

/**
 * As partes da dupla: a chave principal segue até a grande final (quem vence
 * a final dela vai para a direita, na mesma grade); a repescagem fica à
 * parte.
 */
const DOUBLE: Part[][] = [["winners", "grand_final"], ["losers"]];

/**
 * Chave inteira: na simples, uma grade de rodadas; na dupla, a chave principal
 * (com a grande final) e a repescagem no `ContentSwitcher`. Abre na parte da
 * partida que espera quem vê, se ela estiver na repescagem. Sem nenhuma série
 * na repescagem (dupla com 2 inscritos), só a grade da chave principal.
 */
export function BracketView({ tenant, tournament, bracket }: Props) {
  const t = useTranslations("Bracket");
  const single = isSingle(bracket.format);
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
  if (!bracket.matches.some((m) => m.part === "losers")) return grid(DOUBLE[0]);

  const open = bracket.matches.find(
    (m) => m.yours && (m.state === "ready" || m.state === "pending"),
  );
  return (
    <MatchViews
      label={t("partsLabel")}
      defaultIndex={Math.max(
        0,
        DOUBLE.findIndex((parts) => open && parts.includes(open.part)),
      )}
      views={DOUBLE.map((parts) => ({
        key: parts[0],
        label: t(`parts.${parts[0]}`),
        content: grid(parts),
      }))}
    />
  );
}
