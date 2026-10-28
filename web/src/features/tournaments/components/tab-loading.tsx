import { Stack } from "@carbon/react";
import type { ReactNode } from "react";

import styles from "./tab-loading.module.scss";

type Props = {
  /** O que está carregando, lido pelo leitor de tela. */
  label: string;
  /** Os esqueletos na forma da aba. */
  children: ReactNode;
};

/**
 * Uma aba do torneio carregando, dentro da casca (cabeçalho e abas já
 * prontos): o "Carregando…" só para leitor de tela e os esqueletos com a
 * forma da aba, empilhados como os blocos dela.
 */
export function TabLoading({ label, children }: Props) {
  return (
    <Stack gap={7}>
      <span role="status" className={styles.label}>
        {label}
      </span>
      {children}
    </Stack>
  );
}
