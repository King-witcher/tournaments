import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import type { Tiebreaker } from "../lib/tiebreakers";
import { AutoRulesBlock } from "./auto-rules-block";
import { AutoRulesList } from "./auto-rules-list";

type Props = {
  format: "round_robin" | "swiss";
  even: boolean;
  /** Os desempates da fase que contam, na ordem dela. */
  tiebreakers: Tiebreaker[];
  /** Quem decide o empate que sobra: o último item da lista. */
  finalTie: components["schemas"]["FinalTie"];
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
 * chave ou a decisão da organização por último.
 */
export function AutoRulesTable({ format, even, tiebreakers, finalTie }: Props) {
  const t = useTranslations("Tournament.autoRules.table");
  const last = finalTie === "organizer" ? t("organizer") : t("seed");
  const items = [...tiebreakers.map((x) => t(KEY[x])), last];
  return (
    <AutoRulesBlock title={t("title")}>
      <p>
        {format === "swiss"
          ? t("points")
          : `${t(even ? "balanceEven" : "balance")} ${t("balanceOrder")}`}
      </p>
      <AutoRulesList ordered items={items.map((item) => [item])} />
    </AutoRulesBlock>
  );
}
