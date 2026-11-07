import { useTranslations } from "next-intl";

import { isEvenSeries } from "../lib/format";
import type { StageRead } from "../lib/stage-values";
import { countingTiebreakers, stageTiebreakers } from "../lib/tiebreakers";
import { AutoRulesBlock } from "./auto-rules-block";
import { AutoRulesList } from "./auto-rules-list";

type Props = {
  stage: StageRead;
  /** Sem o título do trecho: o conteúdo vai dentro do bloco de uma fase. */
  bare?: boolean;
};

/** O texto de cada desempate na lista. */
const KEY = {
  head_to_head: "headToHead",
  map_diff: "games",
  frag_diff: "frags",
  buchholz: "buchholz",
  lower_rating: "lowerRating",
} as const;

/**
 * Todos contra todos e suíço: o saldo de séries (com o empate só se a série pode
 * empatar) ou os pontos, e os desempates da fase, na ordem, com a ordem da
 * chave ou a decisão da organização por último. Nada nas outras fases.
 */
export function AutoRulesTable({ stage, bare }: Props) {
  const t = useTranslations("Tournament.autoRules.table");
  if (stage.format !== "round_robin" && stage.format !== "swiss") return null;
  const tiebreakers = countingTiebreakers(
    stageTiebreakers(stage) ?? [],
    stage.format,
    stage.best_of,
  );
  const last = stage.final_tie === "organizer" ? t("organizer") : t("seed");
  const items = [...tiebreakers.map((x) => t(KEY[x])), last];
  const content = (
    <>
      <p>
        {stage.format === "swiss"
          ? t("points")
          : `${t(isEvenSeries(stage.best_of) ? "balanceEven" : "balance")} ${t("balanceOrder")}`}
      </p>
      <AutoRulesList ordered items={items.map((item) => [item])} />
    </>
  );
  return bare ? (
    content
  ) : (
    <AutoRulesBlock title={t("title")}>{content}</AutoRulesBlock>
  );
}
