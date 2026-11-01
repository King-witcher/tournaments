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
  /** Edição: as abas, com a primeira (nome, descrição e regras) aberta. */
  edit?: boolean;
};

/**
 * A página do formulário de torneio carregando, na forma dela: na criação,
 * as seções à vista (no Formato, modalidade, vagas e chave) e os seis grupos
 * fechados; na edição, as abas com a primeira (nome, descrição e regras). Os
 * botões no fim, como no `FormActions`.
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
              <FormSectionSkeleton>
                <TextInputSkeleton />
                <TextAreaSkeleton />
              </FormSectionSkeleton>
              <FormSectionSkeleton>
                <TextAreaSkeleton />
              </FormSectionSkeleton>
            </>
          ) : (
            <>
              <FormSectionSkeleton>
                <TextInputSkeleton />
                <TextInputSkeleton />
                <TextInputSkeleton />
                <TextAreaSkeleton />
              </FormSectionSkeleton>
              <FormSectionSkeleton>
                <RadioButtonSkeleton />
                <TextInputSkeleton />
                <RadioButtonSkeleton />
                <SelectSkeleton />
              </FormSectionSkeleton>
              <AccordionSkeleton count={6} open={false} />
            </>
          )}
          <div>
            <ButtonSkeleton />
            <ButtonSkeleton />
            {!edit && <ButtonSkeleton />}
          </div>
        </Stack>
      </div>
    </>
  );
}
