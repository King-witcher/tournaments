import { getTranslations } from "next-intl/server";

import { ButtonLink } from "@/shared/ui/button-link";
import { Column } from "@/shared/ui/carbon/grid";
import { SIGNUP_HREF } from "../lib/signup-href";
import { LandingBand } from "./landing-band";
import styles from "./landing-closing.module.scss";

const TITLE_ID = "landing-closing";

/** O fim da apresentação: criar conta de novo, para quem leu até aqui não voltar ao topo. */
export async function LandingClosing() {
  const t = await getTranslations("Landing.closing");

  return (
    <LandingBand labelledBy={TITLE_ID}>
      <Column sm={4} md={8} lg={10} className={styles.closing}>
        <h2 id={TITLE_ID} className={styles.title}>
          {t("title")}
        </h2>
        <ButtonLink href={SIGNUP_HREF} size="lg">
          {t("action")}
        </ButtonLink>
      </Column>
    </LandingBand>
  );
}
