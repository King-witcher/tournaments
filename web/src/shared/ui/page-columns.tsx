import { Column, Grid, Stack } from "@carbon/react";
import type { ReactNode } from "react";

/**
 * O conteúdo de uma página `wide` em duas colunas do `Grid` do Carbon: a
 * principal e, à direita, uma estreita, para o que é atalho e não deve
 * empurrar o resto para baixo. Abaixo do `lg`, uma embaixo da outra. Vai
 * dentro de `Page`.
 */
export function PageColumns({
  main,
  aside,
}: {
  main: ReactNode;
  aside: ReactNode;
}) {
  return (
    <Grid withRowGap>
      <Column sm={4} md={8} lg={11}>
        <Stack gap={7}>{main}</Stack>
      </Column>
      <Column sm={4} md={8} lg={5}>
        <Stack gap={7}>{aside}</Stack>
      </Column>
    </Grid>
  );
}
