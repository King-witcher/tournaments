"use client";

import { Form, Stack } from "@carbon/react";
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
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { Notice } from "@/shared/ui/notice";
import { TextInput } from "@/shared/ui/text-input";
import { type AuthField, errorField } from "../lib/auth-field";

/** O nome salvo, para o "Salvo." sumir quando ele é editado de novo. */
type Result = FormResult<AuthField> & { savedName?: string };

/** Nome exibido da conta; salvar atualiza o cabeçalho (`router.refresh`). */
export function DisplayNameForm({ name }: { name: string }) {
  const t = useTranslations("Account.name");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const prefix = useId();
  const { ref, canSubmit } = useFormValidity();
  const [value, setValue] = useState(name);

  const [result, submit, pending] = useActionState<Result>(async () => {
    const outcome = await callApi(
      api.PATCH("/me", { body: { display_name: value } }),
    );
    if (!outcome.ok) {
      return {
        error: errorMessage(outcome.code, outcome.params),
        field: errorField(outcome.code),
      };
    }
    setValue(outcome.data.display_name);
    router.refresh();
    return { saved: true, savedName: outcome.data.display_name };
  }, {});
  useFocusInvalid(prefix, result);
  const general = formError(result);
  const saved = result.saved && !pending && result.savedName === value;

  return (
    <Form ref={ref} action={submit}>
      <Stack gap={6}>
        <TextInput
          id={fieldId(prefix, "name")}
          labelText={t("label")}
          helperText={t("hint")}
          required
          maxLength={32}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          {...invalidProps(result, "name")}
        />
        {general && <FormError>{general}</FormError>}
        {saved && <Notice kind="success">{t("saved")}</Notice>}
        <FormActions pending={pending} disabled={!canSubmit} />
      </Stack>
    </Form>
  );
}
