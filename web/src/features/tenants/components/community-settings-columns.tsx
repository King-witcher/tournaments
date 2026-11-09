import type { ReactNode } from "react";

import { Column, Grid } from "@/shared/ui/carbon/grid";
import { Stack } from "@/shared/ui/carbon/stack";
import { getTenant } from "../server";
import styles from "./community-settings-columns.module.scss";

type Props = {
  slug: string;
  /** O que o dono decide sobre a comunidade e o apelido de quem vê. */
  identity: ReactNode;
  /** Os avisos no Discord: só do dono. */
  discord: ReactNode;
  /** A zona de perigo, por último. */
  danger: ReactNode;
};

/**
 * As Configurações da comunidade em duas metades do `lg` para cima, como a
 * seção Geral das configurações do torneio: à esquerda os formulários curtos
 * e o apelido de quem vê, à direita (só o dono) o Discord, a seção mais
 * longa; cada campo na largura de leitura. A zona de perigo fecha a página,
 * na metade da esquerda.
 * Abaixo do `lg`, tudo numa coluna, na mesma ordem. Fica em `<Suspense>`.
 */
export async function CommunitySettingsColumns({
  slug,
  identity,
  discord,
  danger,
}: Props) {
  const loaded = await getTenant(slug);
  if (loaded.kind !== "ok") return null;
  const owner = loaded.tenant.role === "owner";

  return (
    <Stack gap={7}>
      <Grid>
        <Column sm={4} md={8} lg={8}>
          <Stack gap={7}>{identity}</Stack>
        </Column>
        {owner && (
          <Column sm={4} md={8} lg={8} className={styles.second}>
            {discord}
          </Column>
        )}
      </Grid>
      {danger}
    </Stack>
  );
}
