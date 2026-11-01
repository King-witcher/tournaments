import { Column, Grid } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { LandingBand } from "./landing-band";
import styles from "./landing-features.module.scss";

const TITLE_ID = "landing-features";
const FEATURES = ["formats", "teams", "notices", "languages"] as const;

/** O que mais pesa para quem organiza, em quatro colunas curtas. */
export async function LandingFeatures() {
  const t = await getTranslations("Landing.features");

  return (
    <LandingBand labelledBy={TITLE_ID} tone="layer">
      <Column sm={4} md={8} lg={4}>
        <h2 id={TITLE_ID} className={styles.title}>
          {t("title")}
        </h2>
      </Column>
      <Column sm={4} md={8} lg={12}>
        <Grid as="ul" withRowGap>
          {FEATURES.map((feature) => (
            <Column
              as="li"
              key={feature}
              sm={4}
              md={4}
              lg={3}
              className={styles.feature}
            >
              <h3 className={styles.featureTitle}>{t(`${feature}.title`)}</h3>
              <p className={styles.body}>{t(`${feature}.body`)}</p>
            </Column>
          ))}
        </Grid>
      </Column>
    </LandingBand>
  );
}
