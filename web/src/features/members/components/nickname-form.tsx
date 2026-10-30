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

/** O apelido que a API gravou, para dizer "salvo" enquanto o campo não muda. */
type Result = FormResult<"nickname"> & { savedNickname?: string };

/** O código que é erro do próprio campo (vai no `invalidText`, não acima dos botões). */
const FIELD_ERROR = "member.nickname_invalid";

/** O apelido de quem vê nesta comunidade, na aba Configurações. */
export function NicknameForm({
  slug,
  nickname,
}: {
  slug: string;
  nickname: string;
}) {
  const t = useTranslations("Tenant.nickname");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const prefix = useId();
  const [value, setValue] = useState(nickname);
  const { ref, canSubmit } = useFormValidity();

  const [result, submit, pending] = useActionState<Result>(async () => {
    const outcome = await callApi(
      api.PATCH("/tenants/{slug}/me", {
        params: { path: { slug } },
        body: { nickname: value },
      }),
    );
    if (!outcome.ok) {
      return {
        error: errorMessage(outcome.code, outcome.params),
        field: outcome.code === FIELD_ERROR ? "nickname" : undefined,
      };
    }
    setValue(outcome.data.nickname);
    router.refresh();
    return { saved: true, savedNickname: outcome.data.nickname };
  }, {});
  useFocusInvalid(prefix, result);
  const general = formError(result);
  const saved = result.saved && !pending && result.savedNickname === value;

  return (
    <Form ref={ref} action={submit}>
      <Stack gap={6}>
        <TextInput
          id={fieldId(prefix, "nickname")}
          labelText={t("label")}
          name="nickname"
          required
          maxLength={32}
          autoComplete="nickname"
          value={value}
          onChange={(event) => setValue(event.target.value)}
          {...invalidProps(result, "nickname")}
        />
        {general && <FormError>{general}</FormError>}
        {saved && <Notice kind="success">{t("saved")}</Notice>}
        <FormActions pending={pending} disabled={!canSubmit} />
      </Stack>
    </Form>
  );
}
