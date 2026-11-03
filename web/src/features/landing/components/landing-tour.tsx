import { Column } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { LandingBand } from "./landing-band";
import styles from "./landing-tour.module.scss";
import { TourPanel } from "./tour-panel";
import { TourTabs } from "./tour-tabs";

const TITLE_ID = "landing-tour";
const PARTS = ["veto", "draft", "result", "ranking"] as const;

/** O que acontece numa série e depois dela, em abas com a captura de cada parte. */
export async function LandingTour() {
  const t = await getTranslations("Landing.tour");

  return (
    <LandingBand labelledBy={TITLE_ID}>
      <Column sm={4} md={8} lg={16}>
        <h2 id={TITLE_ID} className={styles.title}>
          {t("title")}
        </h2>
        <TourTabs
          label={t("title")}
          tabs={PARTS.map((part) => ({
            key: part,
            label: t(`${part}.tab`),
            content: <TourPanel part={part} />,
          }))}
        />
      </Column>
    </LandingBand>
  );
}
