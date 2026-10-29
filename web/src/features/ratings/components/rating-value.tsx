import { useTranslations } from "next-intl";

import { StatusTag } from "@/shared/ui/status-tag";
import styles from "./rating-value.module.scss";

type Props = { rating: number; provisional: boolean };

/** O rating e, com poucos mapas, a marca "provisório" ao lado. */
export function RatingValue({ rating, provisional }: Props) {
  const t = useTranslations("Ratings");
  return (
    <span className={styles.value}>
      <span className={styles.number}>{rating}</span>
      {provisional && (
        <StatusTag status="neutral" size="sm">
          {t("provisional")}
        </StatusTag>
      )}
    </span>
  );
}
