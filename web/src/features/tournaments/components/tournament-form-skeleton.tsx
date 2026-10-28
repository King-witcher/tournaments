import {
  ButtonSkeleton,
  NumberInputSkeleton,
  SelectSkeleton,
  Stack,
  TextAreaSkeleton,
  TextInputSkeleton,
} from "@carbon/react";

import { FormSectionSkeleton, PageHeaderSkeleton } from "@/shared/ui/skeleton";

type Props = {
  /** O que está carregando, lido pelo leitor de tela. */
  label: string;
  /** Edição: sem o endereço e com a seção da tradução. */
  edit?: boolean;
};

/**
 * A página do formulário de torneio carregando, na forma dela: o cabeçalho
 * com a trilha e as seções de campos de formas diferentes (por isso monta as
 * seções com `FormSectionSkeleton`, e não com o `FormSkeleton`) com os botões
 * no fim.
 */
export function TournamentFormSkeleton({ label, edit = false }: Props) {
  return (
    <>
      <PageHeaderSkeleton label={label} />
      <div aria-hidden>
        <Stack gap={7}>
          <FormSectionSkeleton>
            <TextInputSkeleton />
            {!edit && <TextInputSkeleton />}
            <SelectSkeleton />
          </FormSectionSkeleton>
          <FormSectionSkeleton>
            <SelectSkeleton />
            <SelectSkeleton />
            <SelectSkeleton />
          </FormSectionSkeleton>
          <FormSectionSkeleton>
            <NumberInputSkeleton />
            <TextInputSkeleton />
            <SelectSkeleton />
          </FormSectionSkeleton>
          <FormSectionSkeleton>
            <TextAreaSkeleton />
            <TextAreaSkeleton />
          </FormSectionSkeleton>
          {edit && (
            <FormSectionSkeleton>
              <TextInputSkeleton />
              <TextAreaSkeleton />
            </FormSectionSkeleton>
          )}
          {/* Cancelar e Salvar, lado a lado como no FormActions. */}
          <div>
            <ButtonSkeleton />
            <ButtonSkeleton />
          </div>
        </Stack>
      </div>
    </>
  );
}
