import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { useFormatSeconds } from "../hooks/use-format-seconds";
import { useRuleSentences } from "../hooks/use-rule-sentences";
import { finalsBestOf } from "../lib/format";
import { AutoRulesBlock } from "./auto-rules-block";
import { AutoRulesList } from "./auto-rules-list";
import { Sentences } from "./sentences";

type Props = { tournament: components["schemas"]["Tournament"] };

/**
 * O que acontece em cada série antes de jogar: o "Pronto" (com alguma fase
 * no site), como sai o mapa e os campeões, em passos numerados, e embaixo o
 * tempo de cada escolha, que vale para todos os passos. Sem fase nenhuma
 * (mapa combinado e campeões fora do site), em parágrafos, porque não há
 * passo no site.
 */
export function AutoRulesSeries({ tournament }: Props) {
  const t = useTranslations("Tournament.autoRules.series");
  const formatSeconds = useFormatSeconds();
  const rules = useRuleSentences();
  const tdm = tournament.mode === "tdm";
  const { map_mode: mapMode, champion_phase: championPhase } = tournament.veto;
  const [stage] = tournament.stages;
  // Tudo numa partida só: o veto não tem pick, e não há o que repetir.
  const oneGame = Math.max(stage.best_of, finalsBestOf(stage) ?? 0) === 1;
  // Sem fase nenhuma, a série não pede "Pronto" e não há veto no site.
  const phases = mapMode !== "off" || championPhase;
  // O sorteio do mapa não tem escolha; o resto tem, com tempo.
  const choices = mapMode === "veto" || mapMode === "vote" || championPhase;
  const mapKey =
    oneGame && mapMode === "veto"
      ? "vetoOne"
      : oneGame && mapMode === "off"
        ? "offOne"
        : mapMode;
  const maps = t(`${tdm ? "mapsTdm" : "maps"}.${mapKey}`);
  const champions = rules.champions(tournament);

  if (!phases) {
    return (
      <AutoRulesBlock title={t("title")}>
        <p>{maps}</p>
        <p>
          <Sentences>{champions}</Sentences>
        </p>
      </AutoRulesBlock>
    );
  }
  return (
    <AutoRulesBlock title={t("title")}>
      <AutoRulesList
        ordered
        items={[
          [
            t(tdm ? "readyTdm" : "ready"),
            choices && t(tdm ? "vetoStartsTdm" : "vetoStarts"),
          ],
          [maps],
          champions,
        ]}
      />
      {choices && (
        <p>
          {t("timer", { time: formatSeconds(tournament.veto.step_seconds) })}
        </p>
      )}
    </AutoRulesBlock>
  );
}
