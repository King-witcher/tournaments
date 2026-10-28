import {
  ButtonSkeleton,
  NumberInputSkeleton,
  SelectSkeleton,
  SkeletonText,
  Stack,
  TextAreaSkeleton,
  TextInputSkeleton,
} from "@carbon/react";
import type { ReactNode } from "react";

import { PageHeaderSkeleton } from "@/shared/ui/skeleton";

type Props = {
  /** O que está carregando, lido pelo leitor de tela. */
  label: string;
  /** Edição: sem o endereço e com a seção da tradução. */
  edit?: boolean;
};

/**
 * A página do formulário de torneio carregando, na forma dela: o cabeçalho
 * com a trilha e os grupos de campos com os botões no fim.
 */
export function TournamentFormSkeleton({ label, edit = false }: Props) {
  return (
    <>
      <PageHeaderSkeleton label={label} />
      <div aria-hidden>
        <Stack gap={7}>
          <Group>
            <TextInputSkeleton />
            {!edit && <TextInputSkeleton />}
            <SelectSkeleton />
          </Group>
          <Group>
            <SelectSkeleton />
            <SelectSkeleton />
            <SelectSkeleton />
          </Group>
          <Group>
            <NumberInputSkeleton />
            <TextInputSkeleton />
            <SelectSkeleton />
          </Group>
          <Group>
            <TextAreaSkeleton />
            <TextAreaSkeleton />
          </Group>
          {edit && (
            <Group>
              <TextInputSkeleton />
              <TextAreaSkeleton />
            </Group>
          )}
          {/* Cancelar e Salvar, lado a lado como no ButtonSet. */}
          <div>
            <ButtonSkeleton />
            <ButtonSkeleton />
          </div>
        </Stack>
      </div>
    </>
  );
}

/** A legenda de um grupo e os campos dele. */
function Group({ children }: { children: ReactNode }) {
  return (
    <Stack gap={6}>
      <SkeletonText heading width="40%" />
      {children}
    </Stack>
  );
}
