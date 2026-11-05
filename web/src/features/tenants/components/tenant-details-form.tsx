"use client";

import { useTranslations } from "next-intl";
import { useActionState, useId, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { useFocusInvalid } from "@/shared/hooks/use-focus-invalid";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import {
  type FormResult,
  fieldId,
  formError,
  invalidProps,
} from "@/shared/lib/form-result";
import { Form } from "@/shared/ui/carbon/form";
import { Stack } from "@/shared/ui/carbon/stack";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { Notice } from "@/shared/ui/notice";
import { TextArea } from "@/shared/ui/text-area";
import { TextInput } from "@/shared/ui/text-input";

type Field = "name" | "description";

/** O que a API gravou, para dizer "salvo" enquanto os campos não mudam. */
type Result = FormResult<Field> & {
  savedName?: string;
  savedDescription?: string;
};

/** Os erros que são de um campo: vão no `invalidText` dele, não acima dos botões. */
const FIELD_OF_ERROR: Partial<Record<string, Field>> = {
  "tenant.name_invalid": "name",
  "tenant.description_invalid": "description",
};

type Props = {
  slug: string;
  name: string;
  /** Vazia sem descrição. */
  description: string;
};

/** O nome e a descrição da comunidade, em texto simples (as quebras de linha ficam). */
export function TenantDetailsForm({ slug, name, description }: Props) {
  const t = useTranslations("Tenant.details");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const prefix = useId();
  const [nameValue, setNameValue] = useState(name);
  const [descriptionValue, setDescriptionValue] = useState(description);
  const { ref, canSubmit } = useFormValidity();

  const [result, submit, pending] = useActionState<Result>(async () => {
    const outcome = await callApi(
      api.PATCH("/tenants/{slug}", {
        params: { path: { slug } },
        body: { name: nameValue, description: descriptionValue },
      }),
    );
    if (!outcome.ok) {
      return {
        error: errorMessage(outcome.code, outcome.params),
        field: FIELD_OF_ERROR[outcome.code],
      };
    }
    const savedName = outcome.data.name;
    const savedDescription = outcome.data.description ?? "";
    setNameValue(savedName);
    setDescriptionValue(savedDescription);
    router.refresh();
    return { saved: true, savedName, savedDescription };
  }, {});
  useFocusInvalid(prefix, result);
  const general = formError(result);
  const saved =
    result.saved &&
    !pending &&
    result.savedName === nameValue &&
    result.savedDescription === descriptionValue;

  return (
    <Form ref={ref} action={submit}>
      <Stack gap={6}>
        <TextInput
          id={fieldId(prefix, "name")}
          labelText={t("name")}
          name="name"
          required
          maxLength={64}
          autoComplete="off"
          value={nameValue}
          onChange={(event) => setNameValue(event.target.value)}
          {...invalidProps(result, "name")}
        />
        <TextArea
          id={fieldId(prefix, "description")}
          labelText={t("description")}
          helperText={t("descriptionHint")}
          name="description"
          maxLength={2000}
          value={descriptionValue}
          onChange={(event) => setDescriptionValue(event.target.value)}
          {...invalidProps(result, "description")}
        />
        {general && <FormError>{general}</FormError>}
        {saved && <Notice kind="success">{t("saved")}</Notice>}
        <FormActions pending={pending} disabled={!canSubmit} />
      </Stack>
    </Form>
  );
}
