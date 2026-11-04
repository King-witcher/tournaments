import { Column, Grid } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { Link } from "@/shared/ui/link";
import { ProductShot } from "./product-shot";
import styles from "./tour-panel.module.scss";

type Props = { part: "veto" | "draft" | "result" | "ranking" };

/**
 * Uma aba do tour: o texto e a captura, lado a lado a partir do xlg. Antes,
 * a captura vem embaixo, na largura toda: ao lado do texto ela encolheria
 * até o texto dela não se ler. Nas abas do veto e do draft, o link para a sala
 * de veto, que dá para usar sem conta, na hora em que se vê o que ela é.
 */
export async function TourPanel({ part }: Props) {
  const t = await getTranslations(`Landing.tour.${part}`);
  const tour = await getTranslations("Landing.tour");

  return (
    <Grid>
      <Column sm={4} md={8} lg={10} xlg={4} className={styles.text}>
        <h3 className={styles.title}>{t("title")}</h3>
        <p className={styles.body}>{t("body")}</p>
        {(part === "veto" || part === "draft") && (
          <Link href="/veto">{tour("room")}</Link>
        )}
      </Column>
      <Column sm={4} md={8} lg={16} xlg={12}>
        <ProductShot name={part} alt={t("shot")} />
      </Column>
    </Grid>
  );
}
