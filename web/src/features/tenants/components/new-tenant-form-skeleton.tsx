import { ButtonSkeleton, Stack, TextInputSkeleton } from "@carbon/react";

/** O formulário de criar comunidade carregando: os dois campos e o botão. */
export function NewTenantFormSkeleton() {
  return (
    <div aria-hidden>
      <Stack gap={6}>
        <TextInputSkeleton />
        <TextInputSkeleton />
        <ButtonSkeleton />
      </Stack>
    </div>
  );
}
