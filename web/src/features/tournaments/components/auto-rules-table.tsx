import { useTranslations } from "next-intl";

import { AutoRulesBlock } from "./auto-rules-block";
import { AutoRulesList } from "./auto-rules-list";

type Props = { format: "round_robin" | "swiss" };

/** Todos contra todos e suíço: os pontos e os desempates, na ordem. */
export function AutoRulesTable({ format }: Props) {
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
      <p>{t("points")}</p>
      <AutoRulesList ordered items={tiebreaks.map((tiebreak) => [tiebreak])} />
    </AutoRulesBlock>
  );
}
