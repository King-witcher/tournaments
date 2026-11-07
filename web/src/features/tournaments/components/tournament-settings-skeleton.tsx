import { AccordionSkeleton } from "@/shared/ui/carbon/accordion";
import { Column, Grid } from "@/shared/ui/carbon/grid";
import { TabLoading } from "@/shared/ui/skeleton";
import { SETTINGS_SECTIONS } from "../lib/settings-sections";

type Props = {
  /** O que está carregando, lido pelo leitor de tela. */
  label: string;
};

/**
 * A aba Configurações carregando, na forma dela: o acordeão fechado, um item
 * por seção, na largura toda.
 */
export function TournamentSettingsSkeleton({ label }: Props) {
  return (
    <TabLoading label={label}>
      <Grid>
        <Column sm={4} md={8} lg={16}>
          <div aria-hidden>
            <AccordionSkeleton count={SETTINGS_SECTIONS.length} open={false} />
          </div>
        </Column>
      </Grid>
    </TabLoading>
  );
}
