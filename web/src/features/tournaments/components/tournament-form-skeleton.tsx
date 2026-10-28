import {
  AccordionSkeleton,
  ButtonSkeleton,
  RadioButtonSkeleton,
  SelectSkeleton,
  Stack,
  TextAreaSkeleton,
  TextInputSkeleton,
} from "@carbon/react";

import {
  FormSectionSkeleton,
  PageHeaderSkeleton,
  TabsSkeleton,
} from "@/shared/ui/skeleton";

type Props = {
  /** O que está carregando, lido pelo leitor de tela. */
  label: string;
  /** Edição: as abas, com a primeira (nome e textos) aberta. */
  edit?: boolean;
};

/**
 * A página do formulário de torneio carregando, na forma dela: na criação,
 * as seções à vista, os dois grupos fechados e os textos; na edição, as abas
 * com a primeira. Os botões no fim, como no `FormActions`.
 */
export function TournamentFormSkeleton({ label, edit = false }: Props) {
  return (
    <>
      <PageHeaderSkeleton label={label} />
      <div aria-hidden>
        <Stack gap={7}>
          {edit ? (
            <>
              <TabsSkeleton />
              <TextInputSkeleton />
              <FormSectionSkeleton>
                <TabsSkeleton />
                <TextAreaSkeleton />
                <TextAreaSkeleton />
              </FormSectionSkeleton>
            </>
          ) : (
            <>
              <FormSectionSkeleton>
                <TextInputSkeleton />
                <TextInputSkeleton />
                <TextInputSkeleton />
              </FormSectionSkeleton>
              <FormSectionSkeleton>
                <RadioButtonSkeleton />
                <SelectSkeleton />
              </FormSectionSkeleton>
              <AccordionSkeleton count={2} open={false} />
              <FormSectionSkeleton>
                <TabsSkeleton />
                <TextAreaSkeleton />
              </FormSectionSkeleton>
            </>
          )}
          <div>
            <ButtonSkeleton />
            <ButtonSkeleton />
          </div>
        </Stack>
      </div>
    </>
  );
}
