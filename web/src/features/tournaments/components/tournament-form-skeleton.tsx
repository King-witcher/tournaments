import { AccordionSkeleton } from "@/shared/ui/carbon/accordion";
import { ButtonSkeleton } from "@/shared/ui/carbon/button";
import { RadioButtonSkeleton } from "@/shared/ui/carbon/radio-button";
import { SelectSkeleton } from "@/shared/ui/carbon/select";
import { Stack } from "@/shared/ui/carbon/stack";
import { TextAreaSkeleton } from "@/shared/ui/carbon/text-area";
import { TextInputSkeleton } from "@/shared/ui/carbon/text-input";
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
