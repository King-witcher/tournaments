import { useLocale, useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { isEvenSeries } from "@/features/tournaments/lib/format";
import { stageTiebreakers } from "@/features/tournaments/lib/tiebreakers";
import type { components } from "@/shared/api/schema";
import { fragsText } from "../lib/score";
import { StandingDiff } from "./standing-diff";

type Schemas = components["schemas"];

/** As colunas de número da classificação, além dos pontos do suíço. */
export type NumberColumn = "record" | "maps" | "frags" | "buchholz";

/**
 * A ordem das colunas de número: as séries (o primeiro critério), os critérios
 * de desempate da fase que têm coluna, na ordem dela, e por último o que não
 * desempata. Partidas somem na md1 (repetiriam as séries); o Buchholz, no suíço
 * que não o usa (a API nem o calcula).
 */
export function numberColumns(
  format: Schemas["TournamentBracket"]["format"],
): NumberColumn[] {
  const maps = format.best_of !== 1;
  const criteria = (stageTiebreakers(format) ?? []).flatMap(
    (criterion): NumberColumn[] => {
      if (criterion === "buchholz") return ["buchholz"];
      if (criterion === "frag_diff") return ["frags"];
      return criterion === "map_diff" && maps ? ["maps"] : [];
    },
  );
  const others: NumberColumn[] = maps ? ["maps", "frags"] : ["frags"];
  const rest = others.filter((column) => !criteria.includes(column));
  return ["record", ...criteria, ...rest];
}

/**
 * O conteúdo das colunas de número de uma linha, no padrão único: o saldo com
 * sinal em cima e, embaixo, o recorde da série ou pró – contra (nos frags,
 * marcado como incompleto quando alguma série veio sem eles, porque aí não
 * desempatam; "sem frags" quando nenhuma veio).
 */
export function useStandingNumbers(bestOf: number) {
  const t = useTranslations("Bracket.table");
  const even = isEvenSeries(bestOf);

  return (r: Schemas["TableRow"]): Record<NumberColumn, ReactNode> => {
    const frags = t("detail", {
      won: fragsText(r.frags_for),
      lost: fragsText(r.frags_against),
    });
    return {
      record: (
        <StandingDiff
          value={r.series_diff}
          detail={
            even
              ? t("recordValueDraws", {
                  wins: r.wins,
                  draws: r.draws,
                  losses: r.losses,
                })
              : r.byes > 0
                ? t("recordByes", {
                    wins: r.wins,
                    losses: r.losses,
                    byes: r.byes,
                  })
                : t("recordValue", { wins: r.wins, losses: r.losses })
          }
        />
      ),
      maps: (
        <StandingDiff
          value={r.map_diff}
          detail={t("detail", { won: r.maps_won, lost: r.maps_lost })}
        />
      ),
      // Sem nenhum frag enviado, "sem frags" em vez de 0 – 0 incompleto (com
      // negativos, a soma dá 0 com frags; os dois em 0, quase nunca).
      frags:
        !r.frags_complete && r.frags_for === 0 && r.frags_against === 0 ? (
          t("noFrags")
        ) : (
          <StandingDiff
            value={r.frag_diff}
            detail={r.frags_complete ? frags : t("incomplete", { frags })}
          />
        ),
      buchholz: String(r.buchholz ?? 0),
    };
  };
}

/**
 * O rating do início (a média do time): até 2 casas, sem separador de milhar,
 * como o rating no resto do site.
 */
export function useRatingText() {
  const locale = useLocale();
  const format = new Intl.NumberFormat(locale, {
    maximumFractionDigits: 2,
    useGrouping: false,
  });
  return (r: Schemas["TableRow"]) =>
    r.rating == null ? "" : format.format(r.rating);
}
