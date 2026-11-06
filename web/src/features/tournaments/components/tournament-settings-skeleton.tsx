import { ButtonSkeleton } from "@/shared/ui/carbon/button";
import { Column, Grid } from "@/shared/ui/carbon/grid";
import { RadioButtonSkeleton } from "@/shared/ui/carbon/radio-button";
import { SelectSkeleton } from "@/shared/ui/carbon/select";
import { Stack } from "@/shared/ui/carbon/stack";
import { TextAreaSkeleton } from "@/shared/ui/carbon/text-area";
import { TextInputSkeleton } from "@/shared/ui/carbon/text-input";
import { FormSectionSkeleton, TabLoading } from "@/shared/ui/skeleton";

type Props = {
  /** O que está carregando, lido pelo leitor de tela. */
  label: string;
};

/**
 * A aba Configurações carregando, na forma dela: na coluna estreita, Geral
 * (nome, endereço, descrição e o Salvar) e o começo de Formato.
 */
export function TournamentSettingsSkeleton({ label }: Props) {
  return (
    <TabLoading label={label}>
      <Grid>
        <Column sm={4} md={8} lg={8}>
          <div aria-hidden>
            <Stack gap={9}>
              <FormSectionSkeleton>
                <TextInputSkeleton />
                <TextInputSkeleton />
                <TextAreaSkeleton />
                <ButtonSkeleton />
              </FormSectionSkeleton>
              <FormSectionSkeleton>
                <RadioButtonSkeleton />
                <TextInputSkeleton />
                <RadioButtonSkeleton />
                <SelectSkeleton />
              </FormSectionSkeleton>
            </Stack>
          </div>
        </Column>
      </Grid>
    </TabLoading>
  );
}
