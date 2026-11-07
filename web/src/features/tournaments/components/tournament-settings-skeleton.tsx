import { Column, Grid } from "@/shared/ui/carbon/grid";
import { SettingsSectionSkeleton } from "./settings-section-skeleton";

type Props = {
  /** O que está carregando, lido pelo leitor de tela. */
  label: string;
};

/**
 * As Configurações carregando antes do layout delas (`TournamentSettings`):
 * a seção carregando, na largura toda, na grade que o layout põe.
 */
export function TournamentSettingsSkeleton({ label }: Props) {
  return (
    <Grid>
      <Column sm={4} md={8} lg={16}>
        <SettingsSectionSkeleton label={label} />
      </Column>
    </Grid>
  );
}
