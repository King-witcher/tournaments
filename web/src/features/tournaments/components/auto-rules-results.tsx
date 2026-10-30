import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { useFormatMinutes } from "@/shared/hooks/use-format-minutes";
import { finalsBestOf, GAME_MINUTES, isTable } from "../lib/format";
import { AutoRulesBlock } from "./auto-rules-block";
import { AutoRulesList } from "./auto-rules-list";

type Props = { tournament: components["schemas"]["Tournament"] };

/**
 * Envio e resposta de cada partida: quem envia e onde, a captura de tela do
 * placar (se o torneio exige), os frags (no desempate da tabela), o prazo
 * para confirmar ou contestar, a contestação depois de confirmada e quem
 * decide.
 */
export function AutoRulesResults({ tournament }: Props) {
  const t = useTranslations("Tournament.autoRules.results");
  const formatMinutes = useFormatMinutes();
  const tdm = tournament.mode === "tdm";
  const [stage] = tournament.stages;
  const severalGames = Math.max(stage.best_of, finalsBestOf(stage) ?? 0) > 1;

  return (
    <AutoRulesBlock title={t("title")}>
      <AutoRulesList
        items={[
          [
            t(tdm ? "reportTdm" : "report"),
            isTable(stage) && t("frags"),
            severalGames && t("next"),
          ],
          [tournament.screenshots_required && t("screenshot")],
          [
            t(tdm ? "confirmTdm" : "confirm", {
              window: formatMinutes(tournament.contest_minutes),
              duration: formatMinutes(GAME_MINUTES),
            }),
          ],
          [t(tdm ? "lateTdm" : "late"), t("organizer")],
        ]}
      />
    </AutoRulesBlock>
  );
}
