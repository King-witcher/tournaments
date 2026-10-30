import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { mapsToWin } from "../lib/format";
import { AutoRulesBlock } from "./auto-rules-block";
import { Sentences } from "./sentences";

type Props = { tournament: components["schemas"]["Tournament"] };

/**
 * Quem não aparece e quem sai: avisar a organização, o W.O. e, na tabela, o
 * placar do W.O. e o que acontece com os pontos de quem sai.
 */
export function AutoRulesAbsence({ tournament }: Props) {
  const t = useTranslations("Tournament.autoRules.absence");
  const [stage] = tournament.stages;
  const table = stage.format === "elimination" ? null : stage.format;
  return (
    <AutoRulesBlock title={t("title")}>
      <p>
        <Sentences>
          {[
            t(tournament.mode === "tdm" ? "walkoverTdm" : "walkover"),
            table && t("score", { wins: mapsToWin(stage.best_of) }),
            table && t(`withdrawal.${table}`),
          ]}
        </Sentences>
      </p>
    </AutoRulesBlock>
  );
}
