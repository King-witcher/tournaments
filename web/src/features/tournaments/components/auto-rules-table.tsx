import { useTranslations } from "next-intl";

import { AutoRulesBlock } from "./auto-rules-block";
import { AutoRulesList } from "./auto-rules-list";

type Props = { format: "round_robin" | "swiss"; even: boolean };

/**
 * Todos contra todos e suíço: o saldo de séries (com o empate só se a série pode
 * empatar) ou os pontos, e os desempates, na ordem.
 */
export function AutoRulesTable({ format, even }: Props) {
  const t = useTranslations("Tournament.autoRules.table");
  const tiebreaks = [
    ...(format === "swiss" ? [t("buchholz")] : []),
    t("headToHead"),
    t("games"),
    t("frags"),
    t("seed"),
  ];
  return (
    <AutoRulesBlock title={t("title")}>
      <p>
        {format === "swiss"
          ? t("points")
          : `${t(even ? "balanceEven" : "balance")} ${t("balanceOrder")}`}
      </p>
      <AutoRulesList ordered items={tiebreaks.map((tiebreak) => [tiebreak])} />
    </AutoRulesBlock>
  );
}
