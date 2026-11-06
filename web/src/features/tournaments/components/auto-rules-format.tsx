import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { useRuleSentences } from "../hooks/use-rule-sentences";
import { finalsBestOf, isEvenSeries, mapsToWin } from "../lib/format";
import { AutoRulesBlock } from "./auto-rules-block";
import { Sentences } from "./sentences";

type Props = { tournament: components["schemas"]["Tournament"] };

/**
 * A chave (ou o todos contra todos, o suíço, a disputa de posições): o nome,
 * como se avança ou se cai, o melhor de N das séries e da final e, na série
 * par, o empate e o fim antes de jogar todas.
 */
export function AutoRulesFormat({ tournament }: Props) {
  const t = useTranslations("Tournament.autoRules.format");
  const rules = useRuleSentences();
  const tdm = tournament.mode === "tdm";
  const [stage] = tournament.stages;
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
    <AutoRulesBlock
      title={t(
        stage.format === "elimination" ? "title.elimination" : "title.table",
      )}
    >
      <p>
        <Sentences>{[name, rules.bracketRule(tournament)]}</Sentences>
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
    </AutoRulesBlock>
  );
}
