import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { mapsToWin } from "../lib/format";
import { AutoRulesBlock } from "./auto-rules-block";
import { Sentences } from "./sentences";

type Props = { tournament: components["schemas"]["Tournament"] };

/**
 * Quem não aparece e quem sai: avisar a organização, o W.O. e, na tabela, o
 * placar do W.O. e o que acontece com os pontos de quem sai. Com várias fases
 * de tabela, o placar e a regra de cada uma, sem repetir a que é igual.
 */
export function AutoRulesAbsence({ tournament }: Props) {
  const t = useTranslations("Tournament.autoRules.absence");
  const tables = tournament.stages.flatMap((stage) =>
    stage.format === "round_robin" || stage.format === "swiss"
      ? [
          [
            t("score", { wins: mapsToWin(stage.best_of) }),
            t(`withdrawal.${stage.format}`),
          ],
        ]
      : [],
  );
  // Fases de tabela iguais dizem o mesmo uma vez.
  const sentences = [...new Set(tables.flat())];
  return (
    <AutoRulesBlock title={t("title")}>
      <p>
        <Sentences>
          {[
            t(tournament.mode === "tdm" ? "walkoverTdm" : "walkover"),
            ...sentences,
          ]}
        </Sentences>
      </p>
    </AutoRulesBlock>
  );
}
