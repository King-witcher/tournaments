"use client";

import { Form, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { Notice } from "@/shared/ui/notice";
import { TextInput } from "@/shared/ui/text-input";

type State = {
  /** O apelido que a API gravou, para dizer "salvo" enquanto o campo não muda. */
  saved?: string;
  error?: { code: string; message: string };
};

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
  const [value, setValue] = useState(nickname);

  const [state, submit, pending] = useActionState<State>(async () => {
    const result = await callApi(
      api.PATCH("/tenants/{slug}/me", {
        params: { path: { slug } },
        body: { nickname: value },
      }),
    );
    if (!result.ok) {
      return {
        error: {
          code: result.code,
          message: errorMessage(result.code, result.params),
        },
      };
    }
    setValue(result.data.nickname);
    router.refresh();
    return { saved: result.data.nickname };
  }, {});

  const fieldError =
    state.error?.code === FIELD_ERROR ? state.error.message : undefined;

  return (
    <Form action={submit}>
      <Stack gap={6}>
        <TextInput
          labelText={t("label")}
          helperText={t("hint")}
          name="nickname"
          required
          maxLength={32}
          autoComplete="nickname"
          value={value}
          onChange={(event) => setValue(event.target.value)}
          invalid={fieldError !== undefined}
          invalidText={fieldError}
        />
        {state.error && fieldError === undefined && (
          <FormError>{state.error.message}</FormError>
        )}
        {state.saved !== undefined && state.saved === value && !pending && (
          <Notice kind="success">{t("saved")}</Notice>
        )}
        <FormActions pending={pending} />
      </Stack>
    </Form>
  );
}
