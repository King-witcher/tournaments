import { Stack } from "@carbon/react";
import type { ReactNode } from "react";

import styles from "./account-tab.module.scss";

/**
 * O conteúdo de uma aba da conta: o espaço abaixo das abas (o `PageTabs`
 * tira o padding do painel) e as seções empilhadas. Use também no
 * `loading.tsx` da aba, para o esqueleto ficar no mesmo lugar.
 */
export function AccountTab({ children }: { children: ReactNode }) {
  return (
    <div className={styles.tab}>
      <Stack gap={7}>{children}</Stack>
    </div>
  );
}
