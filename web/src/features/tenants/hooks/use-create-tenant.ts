import { useActionState, useId, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFocusInvalid } from "@/shared/hooks/use-focus-invalid";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import { focusIfLost } from "@/shared/lib/focus";
import type { FormResult } from "@/shared/lib/form-result";
import { slugify } from "@/shared/lib/slugify";
import { TENANT_TITLE_ID } from "../lib/focus";

export type CreateTenantField = "name" | "slug";

/** Os erros que são de um campo: vão no `invalidText` dele, não acima dos botões. */
const FIELD_OF_ERROR: Partial<Record<string, CreateTenantField>> = {
  "tenant.name_invalid": "name",
  "tenant.slug_taken": "slug",
  "tenant.slug_invalid": "slug",
  "tenant.slug_reserved": "slug",
};

/**
 * Criar comunidade, na página e no Modal: nome e endereço, que acompanha o
 * nome até a pessoa editá-lo. Criada, leva a ela, com o foco no título.
 * `submit` é a ação do `useActionState`: o `action` do `<Form>` na página; no
 * Modal, dentro de `startTransition`. O logo fica para Configurações.
 */
export function useCreateTenant() {
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const prefix = useId();
  const { ref: formRef, canSubmit } = useFormValidity();
  const [name, setName] = useState("");
  const [slug, setSlug] = useState("");
  const [slugEdited, setSlugEdited] = useState(false);

  const [result, submit, pending] = useActionState<
    FormResult<CreateTenantField>
  >(async () => {
    const outcome = await callApi(
      api.POST("/tenants", { body: { name, slug } }),
    );
    if (!outcome.ok) {
      return {
        error: errorMessage(outcome.code, outcome.params),
        field: FIELD_OF_ERROR[outcome.code],
      };
    }
    router.push(`/${outcome.data.slug}`);
    focusIfLost(TENANT_TITLE_ID);
    return {};
  }, {});
  useFocusInvalid(prefix, result);

  return {
    formRef,
    /** Nenhum campo recusa o que tem: "Criar" pode ser liberado. */
    canSubmit,
    /** O `useId` dos campos (`fieldId`), para o foco ir ao recusado. */
    prefix,
    name,
    slug,
    changeName: (value: string) => {
      setName(value);
      if (!slugEdited) setSlug(slugify(value));
    },
    changeSlug: (value: string) => {
      setSlugEdited(true);
      setSlug(value);
    },
    result,
    submit,
    pending,
  };
}

export type CreateTenantState = ReturnType<typeof useCreateTenant>;
