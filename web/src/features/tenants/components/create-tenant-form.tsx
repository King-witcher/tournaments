"use client";

import { useTranslations } from "next-intl";

import { formError } from "@/shared/lib/form-result";
import { Form } from "@/shared/ui/carbon/form";
import { Stack } from "@/shared/ui/carbon/stack";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { useCreateTenant } from "../hooks/use-create-tenant";
import { CreateTenantFields } from "./create-tenant-fields";

/**
 * Criar comunidade numa página (`/new`: depois do cadastro e no link direto;
 * de dentro do site, a criação é o `CreateTenantModal`): nome e endereço,
 * "Criar" e "Cancelar".
 */
export function CreateTenantForm() {
  const t = useTranslations("NewTenant");
  const form = useCreateTenant();
  const general = formError(form.result);

  return (
    <Form ref={form.formRef} action={form.submit}>
      <Stack gap={6}>
        <CreateTenantFields form={form} />
        {general && <FormError>{general}</FormError>}
        <FormActions
          submitLabel={t("submit")}
          pending={form.pending}
          disabled={!form.canSubmit}
          cancelHref="/"
        />
      </Stack>
    </Form>
  );
}
