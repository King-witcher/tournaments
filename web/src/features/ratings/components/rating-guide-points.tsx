import { getTranslations } from "next-intl/server";

import { Section } from "@/shared/ui/section";
import { StructuredList } from "@/shared/ui/structured-list";

/**
 * Saídas do Elo do `engine` (`rating/elo.rs`) com K 32. Mudou a fórmula ou o
 * padrão: refaça estes números e os do texto do artigo (`RatingGuide.json`).
 */
const ROWS = [
  { difference: 0, chance: "50%", higher: 16, lower: 16 },
  { difference: 100, chance: "64%", higher: 12, lower: 20 },
  { difference: 200, chance: "76%", higher: 8, lower: 24 },
  { difference: 400, chance: "91%", higher: 3, lower: 29 },
];

/** A parte do artigo do rating que mostra quanto vale uma vitória. */
export async function RatingGuidePoints() {
  const t = await getTranslations("RatingGuide.points");
  return (
    <Section title={t("title")}>
      <p>{t("text")}</p>
      <StructuredList
        label={t("label")}
        condensed
        headers={[
          t("difference"),
          t("chance"),
          t("higherWins"),
          t("lowerWins"),
        ]}
        rows={ROWS.map((row) => ({
          key: String(row.difference),
          cells: [row.difference, row.chance, row.higher, row.lower],
        }))}
      />
      <p>{t("loser")}</p>
      <p>{t("balance")}</p>
    </Section>
  );
}
