import { ButtonSkeleton } from "@/shared/ui/carbon/button";
import { SkeletonText } from "@/shared/ui/carbon/skeleton-text";
import { Stack } from "@/shared/ui/carbon/stack";
import { TextInputSkeleton } from "@/shared/ui/carbon/text-input";
import { FormRow } from "@/shared/ui/form-row";
import { TabLoading } from "@/shared/ui/skeleton";

type Props = {
  /** O que está carregando, lido pelo leitor de tela. */
  label: string;
};

/**
 * Uma seção das Configurações carregando, na forma da página dela: o título e
 * o resumo, uma linha de campos lado a lado, um campo sozinho e o Salvar.
 */
export function SettingsSectionSkeleton({ label }: Props) {
  return (
    <TabLoading label={label}>
      <div aria-hidden>
        <Stack gap={6}>
          <Stack gap={3}>
            <SkeletonText heading width="20%" />
            <SkeletonText width="40%" />
          </Stack>
          <FormRow columns={2}>
            <TextInputSkeleton />
            <TextInputSkeleton />
          </FormRow>
          <FormRow columns={4}>
            <TextInputSkeleton />
          </FormRow>
          <ButtonSkeleton />
        </Stack>
      </div>
    </TabLoading>
  );
}
