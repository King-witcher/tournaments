import { useTranslations } from "next-intl";
import { type ReactNode, useId } from "react";

import styles from "./danger-zone.module.scss";

type Props = {
  /** As ações destrutivas, cada uma numa `Section level={3}`. */
  children: ReactNode;
};

/**
 * O fim de uma página de configurações: as ações que apagam ou não voltam
 * (excluir, transferir a posse, sair), juntas num bloco com borda e título
 * em vermelho, apartado do resto. Cada ação é uma `Section level={3}`; os
 * botões continuam sendo os de perigo (`ConfirmButton`) com a confirmação.
 */
export function DangerZone({ children }: Props) {
  const t = useTranslations("Common");
  const headingId = useId();
  return (
    <section aria-labelledby={headingId} className={styles.zone}>
      <h2 id={headingId} className={styles.title}>
        {t("dangerZone")}
      </h2>
      <div className={styles.body}>{children}</div>
    </section>
  );
}
