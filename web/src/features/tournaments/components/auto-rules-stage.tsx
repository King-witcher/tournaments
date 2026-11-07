import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { useStageEntry } from "../hooks/use-stage-entry";
import type { StageRead } from "../lib/stage-values";
import { AutoRulesBlock } from "./auto-rules-block";
import { AutoRulesFormatText } from "./auto-rules-format-text";
import { AutoRulesTable } from "./auto-rules-table";
import { Sentences } from "./sentences";

type Props = {
  tournament: components["schemas"]["Tournament"];
  stage: StageRead;
};

/**
 * Uma fase de um torneio com várias, em um trecho de título "Fase N": da
 * segunda em diante, quem entra e o check-in; depois o formato, as séries e,
 * nas fases de tabela, a classificação com os desempates dela.
 */
export function AutoRulesStage({ tournament, stage }: Props) {
  const t = useTranslations("Tournament.setup");
  const entry = useStageEntry();

  return (
    <AutoRulesBlock title={t("stage", { number: stage.number })}>
      {stage.number > 1 && (
        <p>
          <Sentences>
            {entry(stage.advance, stage.check_in, stage.number)}
          </Sentences>
        </p>
      )}
      <AutoRulesFormatText tournament={tournament} stage={stage} />
      <AutoRulesTable stage={stage} bare />
    </AutoRulesBlock>
  );
}
