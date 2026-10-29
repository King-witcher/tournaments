import { useTranslations } from "next-intl";

import styles from "./or-divider.module.scss";

/**
 * O "ou" entre as formas de entrar (o formulário e os provedores), com uma
 * linha de cada lado: separa as duas sem parecer que são um passo só.
 */
export function OrDivider() {
  const t = useTranslations("Auth");
  return <p className={styles.divider}>{t("or")}</p>;
}
