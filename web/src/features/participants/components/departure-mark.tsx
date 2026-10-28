import { useTranslations } from "next-intl";

import styles from "./departure-mark.module.scss";

type Props = {
  /** Desistiu do torneio em andamento. */
  withdrawn: boolean;
  /** Saiu da comunidade (ou foi removido). */
  left: boolean;
};

/**
 * Marca discreta de quem não segue no torneio: "saiu" (da comunidade) ou
 * "desistiu". Nada para quem segue.
 */
export function DepartureMark({ withdrawn, left }: Props) {
  const t = useTranslations("Participants.state");
  if (!withdrawn && !left) return null;
  return (
    <span className={styles.mark}>{left ? t("left") : t("withdrawn")}</span>
  );
}
