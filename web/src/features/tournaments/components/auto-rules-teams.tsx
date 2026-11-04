import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { useRuleSentences } from "../hooks/use-rule-sentences";
import { AutoRulesBlock } from "./auto-rules-block";
import { Sentences } from "./sentences";

type Props = { tournament: components["schemas"]["Tournament"] };

/** TDM: o tamanho dos times, os reservas, as trocas durante o torneio e quem age pelo time. */
export function AutoRulesTeams({ tournament }: Props) {
  const t = useTranslations("Tournament.autoRules.teams");
  const rules = useRuleSentences();
  return (
    <AutoRulesBlock title={t("title")}>
      {rules.teams(tournament).map((paragraph) => (
        <p key={paragraph}>{paragraph}</p>
      ))}
      <p>
        <Sentences>{rules.actors(tournament)}</Sentences>
      </p>
    </AutoRulesBlock>
  );
}
