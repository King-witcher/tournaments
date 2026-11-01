import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import type { ProgressStepItem } from "@/shared/ui/progress-steps";
import { type seriesSteps, stepGame } from "../lib/match-steps";

type Detail = components["schemas"]["MatchDetail"];

/**
 * Os passos da série como o indicador os mostra: o nome ("Mapas", "Partida
 * 2") e, embaixo, o placar da partida confirmada, o estado da enviada
 * ("Aguardando"; "Contestada", com o alerta) ou "Agora" no passo atual.
 */
export function useStepItems() {
  const t = useTranslations("Match.steps");
  const tGames = useTranslations("Match.games");
  const gameLabel = (game: Detail["games"][number]) =>
    game.status !== "confirmed"
      ? tGames(`badge.${game.status}`)
      : game.a_frags != null && game.b_frags != null
        ? t("score", { a: game.a_frags, b: game.b_frags })
        : tGames("badge.confirmed");

  return (
    detail: Detail,
    flow: ReturnType<typeof seriesSteps>,
  ): ProgressStepItem[] =>
    flow.steps.map((key, i) => {
      const number = stepGame(key);
      const game = detail.games.find((g) => g.number === number);
      const now = i === flow.current && detail.summary.state === "ready";
      return {
        key,
        label:
          number == null
            ? t(key as "ready" | "maps" | "champions")
            : tGames("game", { number }),
        secondaryLabel: game ? gameLabel(game) : now ? t("now") : undefined,
        invalid: game?.status === "contested",
      };
    });
}
