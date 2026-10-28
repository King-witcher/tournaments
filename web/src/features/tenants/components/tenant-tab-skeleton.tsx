import { Stack } from "@carbon/react";
import type { ReactNode } from "react";

import styles from "./tenant-tab-skeleton.module.scss";

/**
 * Uma aba da comunidade carregando: as peças de esqueleto com a forma dela
 * (`TableSkeleton`, `SectionSkeleton`) e o que está carregando, lido pelo
 * leitor de tela (as peças são só visuais, e o cabeçalho com o "Carregando…"
 * já está na tela quando se troca de aba).
 */
export function TenantTabSkeleton({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <div>
      <span role="status" className={styles.label}>
        {label}
      </span>
      <Stack gap={7}>{children}</Stack>
    </div>
  );
}
