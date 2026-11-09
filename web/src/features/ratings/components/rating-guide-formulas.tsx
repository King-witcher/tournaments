import { getTranslations } from "next-intl/server";
import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";
import { Section } from "@/shared/ui/section";
import styles from "./rating-guide-formulas.module.scss";

const MATH = {
  var: (chunks: ReactNode) => <var>{chunks}</var>,
  sup: (chunks: ReactNode) => <sup>{chunks}</sup>,
  sub: (chunks: ReactNode) => <sub>{chunks}</sub>,
};

const BLOCKS = [
  { title: "chanceTitle", formula: "chance", text: "chanceText" },
  { title: "kTitle", formula: "k", text: "kText" },
  { title: "pointsTitle", formula: "points", text: "pointsText" },
] as const;

/** O fim do artigo do rating: a conta exata, para quem quer refazê-la. */
export async function RatingGuideFormulas() {
  const t = await getTranslations("RatingGuide.formulas");
  return (
    <Section title={t("title")} description={t("description")}>
      {BLOCKS.map((block) => (
        <Section key={block.title} title={t(block.title)} level={3}>
          <p className={cn(styles.math, styles.formula)}>
            {t.rich(block.formula, MATH)}
          </p>
          <p className={styles.math}>{t.rich(block.text, MATH)}</p>
        </Section>
      ))}
      <Section title={t("exampleTitle")} level={3}>
        <p className={styles.math}>{t.rich("example", MATH)}</p>
      </Section>
    </Section>
  );
}
