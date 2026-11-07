import type { ReactNode } from "react";

import { Column, Grid } from "@/shared/ui/carbon/grid";
import { Stack } from "@/shared/ui/carbon/stack";
import styles from "./home-columns.module.scss";

type Props = {
  /** O que vem antes de tudo ("Esperando você"); some quando não desenha nada. */
  lead: ReactNode;
  /** A coluna estreita ("Suas comunidades"); some quando não desenha nada. */
  aside: ReactNode;
  /** O resto da coluna principal, embaixo do `lead`. */
  main?: ReactNode;
};

/**
 * As colunas do início de quem entrou, as do `PageColumns` (11 e 5 do `lg`
 * para cima) com uma diferença: abaixo do `lg`, a coluna estreita entra depois
 * do `lead` e antes do resto. No celular, as comunidades (poucas, e o caminho
 * para tudo) ficam logo depois das tarefas, não no fim de uma lista longa de
 * torneios. `lead` e `aside` são um componente cada (sem nada em volta), para
 * o vazio deles sumir sem deixar espaço.
 */
export function HomeColumns({ lead, aside, main }: Props) {
  return (
    <Grid className={styles.grid}>
      <Column sm={4} md={8} lg={11} className={styles.lead}>
        {lead}
      </Column>
      <Column sm={4} md={8} lg={5} className={styles.aside}>
        {aside}
      </Column>
      {main && (
        <Column sm={4} md={8} lg={11} className={styles.main}>
          <Stack gap={7}>{main}</Stack>
        </Column>
      )}
    </Grid>
  );
}
