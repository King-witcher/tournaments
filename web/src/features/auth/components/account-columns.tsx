import type { ReactNode } from "react";

import { Column, Grid } from "@/shared/ui/carbon/grid";
import { Stack } from "@/shared/ui/carbon/stack";
import styles from "./account-columns.module.scss";

type Props = {
  /** A metade da esquerda: as seções que vêm primeiro. */
  start: ReactNode;
  /** A metade da direita. */
  end: ReactNode;
  /** Por último, na metade da esquerda: a zona de perigo. */
  danger?: ReactNode;
};

/**
 * Uma página da conta em duas metades do `lg` para cima, como as
 * Configurações da comunidade: cada seção na largura de leitura, e as duas
 * metades lado a lado em vez de um bloco estreito com o resto vazio. A zona
 * de perigo fecha a página, na metade da esquerda. Abaixo do `lg`, tudo numa
 * coluna, na mesma ordem. Serve também ao esqueleto da página.
 */
export function AccountColumns({ start, end, danger }: Props) {
  const halves = (
    <Grid>
      <Column sm={4} md={8} lg={8}>
        <Stack gap={9}>{start}</Stack>
      </Column>
      <Column sm={4} md={8} lg={8} className={styles.second}>
        <Stack gap={9}>{end}</Stack>
      </Column>
    </Grid>
  );
  if (!danger) return halves;
  return (
    <Stack gap={9}>
      {halves}
      <Grid>
        <Column sm={4} md={8} lg={8}>
          {danger}
        </Column>
      </Grid>
    </Stack>
  );
}
