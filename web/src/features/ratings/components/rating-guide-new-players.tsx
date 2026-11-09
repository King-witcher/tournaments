import { getTranslations } from "next-intl/server";

import { Section } from "@/shared/ui/section";
import { StructuredList } from "@/shared/ui/structured-list";

/**
 * Saídas de `EloConfig::k` no padrão (`rating/elo.rs`), e metade delas.
 * Mudou a fórmula ou o padrão: refaça estes números e os do texto do artigo
 * (`RatingGuide.json`).
 */
const ROWS = [
  { games: 0, k: 80, win: 40 },
  { games: 5, k: 61, win: 31 },
  { games: 10, k: 50, win: 25 },
  { games: 20, k: 38, win: 19 },
  { games: 40, k: 33, win: 16 },
];

/** A parte do artigo do rating que explica o fator K. */
export async function RatingGuideNewPlayers() {
  const t = await getTranslations("RatingGuide.newPlayers");
  return (
    <Section title={t("title")}>
      <p>{t("why")}</p>
      <p>{t("k")}</p>
      <StructuredList
        label={t("label")}
        condensed
        headers={[t("games"), t("factor"), t("win")]}
        rows={ROWS.map((row) => ({
          key: String(row.games),
          cells: [row.games, row.k, row.win],
        }))}
      />
      <p>{t("after")}</p>
      <p>{t("own")}</p>
      <p>{t("provisional")}</p>
    </Section>
  );
}
