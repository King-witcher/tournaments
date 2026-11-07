import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { useRuleSentences } from "../hooks/use-rule-sentences";
import { finalsBestOf, isEvenSeries, mapsToWin } from "../lib/format";
import type { StageRead } from "../lib/stage-values";
import { Sentences } from "./sentences";

type Props = {
  tournament: components["schemas"]["Tournament"];
  /** A fase de que o texto fala. */
  stage: StageRead;
};

/**
 * O texto de uma fase: o nome do formato, como se avança ou se cai, o melhor
 * de N das séries e da final e, na série par, o empate e o fim antes de jogar
 * todas. Dois parágrafos, para o bloco do trecho ou da fase.
 */
export function AutoRulesFormatText({ tournament, stage }: Props) {
  const t = useTranslations("Tournament.autoRules.format");
  const rules = useRuleSentences();
  const tdm = tournament.mode === "tdm";
  const finals = finalsBestOf(stage) ?? stage.best_of;
  const name =
    stage.format === "elimination"
      ? t(stage.bracket)
      : stage.format === "swiss"
        ? stage.rounds != null
          ? t("swissRounds", { rounds: stage.rounds })
          : t(tdm ? "swissTdm" : "swiss")
        : stage.format === "placement"
          ? t("placement")
          : t(tdm ? "round_robinTdm" : "round_robin");

  return (
    <>
      <p>
        <Sentences>{[name, rules.bracketRule(tournament, stage)]}</Sentences>
      </p>
      <p>
        <Sentences>
          {[
            stage.format === "elimination" && finals !== stage.best_of
              ? t(`finals.${stage.bracket}`, { bestOf: stage.best_of, finals })
              : t("series", { bestOf: stage.best_of }),
            isEvenSeries(stage.best_of) &&
              t("seriesEven", { wins: mapsToWin(stage.best_of) }),
          ]}
        </Sentences>
      </p>
    </>
  );
}
