import { ButtonSet, Column, Grid } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { ButtonLink } from "@/shared/ui/button-link";
import { Link } from "@/shared/ui/link";
import { SIGNUP_HREF } from "../lib/signup-href";
import styles from "./landing-hero.module.scss";
import { ProductShot } from "./product-shot";

const TITLE_ID = "landing-title";

/**
 * A abertura da apresentação: o que o site é, criar conta (primário) e
 * entrar, a sala de veto (um caminho a mais, em link, para quem só quer o veto,
 * sem conta) e a chave de um torneio. O título é o momento expressivo do Carbon
 * (`fluid-heading-05`), fora de qualquer caixa; o resto do site segue no
 * produtivo.
 */
export async function LandingHero() {
  const t = await getTranslations("Landing.hero");

  return (
    <section aria-labelledby={TITLE_ID} className={styles.hero}>
      <Grid>
        {/* Lado a lado a partir do xlg; antes, a captura vem embaixo do
            texto, que no lg teria uma linha por palavra ao lado dela. */}
        <Column sm={4} md={8} lg={10} xlg={6} className={styles.text}>
          <h1 id={TITLE_ID} className={styles.title}>
            {t("title")}
          </h1>
          <p className={styles.lead}>{t("lead")}</p>
          {/* Página inteira: o primário à esquerda, como o Carbon pede. */}
          <ButtonSet className={styles.actions}>
            <ButtonLink href={SIGNUP_HREF} size="lg">
              {t("signup")}
            </ButtonLink>
            <ButtonLink href="/login" kind="secondary" size="lg">
              {t("login")}
            </ButtonLink>
          </ButtonSet>
          <p className={styles.room}>
            {t.rich("room", {
              link: (chunks) => (
                <Link href="/veto" inline>
                  {chunks}
                </Link>
              ),
            })}
          </p>
        </Column>
        <Column sm={4} md={8} lg={16} xlg={10} className={styles.shot}>
          <ProductShot name="bracket" alt={t("shot")} />
        </Column>
      </Grid>
    </section>
  );
}
