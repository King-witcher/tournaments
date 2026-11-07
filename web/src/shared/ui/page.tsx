import type { ReactNode } from "react";

import { Column, Grid } from "./carbon/grid";
import { Stack } from "./carbon/stack";
import styles from "./page.module.scss";

type Props = {
  children: ReactNode;
  /**
   * `narrow` (padrão): texto e formulário, 8 colunas a partir de `lg` (no
   * meio; com o menu lateral, as primeiras), a largura toda abaixo. `wide`:
   * tabela e chave, 16 colunas.
   */
  width?: "narrow" | "wide";
};

/**
 * O conteúdo de uma página, dentro do `Content` da casca: `Grid` do Carbon
 * com a coluna da largura pedida e os blocos (`PageHeader`, `PageTabs`,
 * `Section`) empilhados com o espaçamento padrão.
 */
export function Page({ children, width = "narrow" }: Props) {
  return (
    <Grid className={styles.grid}>
      <Column
        sm={4}
        md={8}
        lg={width === "wide" ? 16 : { span: 8, offset: 4 }}
        className={width === "wide" ? undefined : styles.narrow}
      >
        <Stack gap={7}>{children}</Stack>
      </Column>
    </Grid>
  );
}
