import { getTranslations } from "next-intl/server";

import { Column, Grid } from "@/shared/ui/carbon/grid";
import { LandingBand } from "./landing-band";
import styles from "./landing-steps.module.scss";

const TITLE_ID = "landing-steps";
const STEPS = ["community", "tournament", "play"] as const;

/**
 * Os três passos de quem organiza, em ordem: título à esquerda e os passos
 * em colunas, separados por uma linha (em cima, um embaixo do outro, no
 * celular), como a seção de informação do tutorial do Carbon.
 */
export async function LandingSteps() {
  const t = await getTranslations("Landing.steps");

  return (
    <LandingBand labelledBy={TITLE_ID} tone="layer">
      <Column sm={4} md={8} lg={4}>
        <h2 id={TITLE_ID} className={styles.title}>
          {t("title")}
        </h2>
      </Column>
      <Column sm={4} md={8} lg={12}>
        <Grid as="ol">
          {STEPS.map((step, i) => (
            <Column as="li" key={step} sm={4} md={8} lg={4}>
              <div className={styles.step}>
                {/* A ordem já está na lista; o número é para quem vê. */}
                <span aria-hidden className={styles.number}>
                  {i + 1}
                </span>
                <h3 className={styles.stepTitle}>{t(`${step}.title`)}</h3>
                <p className={styles.body}>{t(`${step}.body`)}</p>
              </div>
            </Column>
          ))}
        </Grid>
      </Column>
    </LandingBand>
  );
}
