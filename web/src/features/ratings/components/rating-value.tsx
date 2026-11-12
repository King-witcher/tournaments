import { useTranslations } from "next-intl";

import styles from "./rating-value.module.scss";

type Props = { rating: number; provisional: boolean };

/**
 * O rating e, com poucos mapas, "provisório" ao lado, em texto secundário:
 * no começo de uma comunidade quase todos são, e a marca não pede ação.
 */
export function RatingValue({ rating, provisional }: Props) {
  const t = useTranslations("Ratings");
  return (
    <span className={styles.value}>
      <span className={styles.number}>{rating}</span>
      {provisional && (
        <span className={styles.provisional}>{t("provisional")}</span>
      )}
    </span>
  );
}
