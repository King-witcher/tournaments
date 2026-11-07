import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { AutoRulesBlock } from "./auto-rules-block";
import { AutoRulesFormatText } from "./auto-rules-format-text";

type Props = { tournament: components["schemas"]["Tournament"] };

/**
 * A chave (ou o todos contra todos, o suíço, a disputa de posições) do torneio
 * de uma fase só. Com várias, cada fase tem o seu trecho (`AutoRulesStage`).
 */
export function AutoRulesFormat({ tournament }: Props) {
  const t = useTranslations("Tournament.autoRules.format");
  const [stage] = tournament.stages;

  return (
    <AutoRulesBlock
      title={t(
        stage.format === "elimination" ? "title.elimination" : "title.table",
      )}
    >
      <AutoRulesFormatText tournament={tournament} stage={stage} />
    </AutoRulesBlock>
  );
}
