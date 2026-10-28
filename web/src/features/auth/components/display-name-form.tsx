"use client";

import { useTranslations } from "next-intl";
import { useActionState, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { Field } from "@/shared/ui/field";
import { Input } from "@/shared/ui/input";

type State = { saved: boolean; error?: string };

/** Nome exibido da conta; salvar atualiza o cabeçalho (`router.refresh`). */
export function DisplayNameForm({ name }: { name: string }) {
  const t = useTranslations("Account.name");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const [value, setValue] = useState(name);

  const [state, submit, pending] = useActionState<State>(
    async () => {
      const result = await callApi(
        api.PATCH("/me", { body: { display_name: value } }),
      );
      if (!result.ok) return { saved: false, error: errorMessage(result.code) };
      setValue(result.data.display_name);
      router.refresh();
      return { saved: true };
    },
    { saved: false },
  );

  return (
    <form action={submit} className="flex flex-col gap-1">
      <Field label={t("label")} hint={t("hint")}>
        <span className="flex gap-2">
          <Input
            required
            maxLength={32}
            value={value}
            onChange={(e) => setValue(e.target.value)}
            className="min-w-0 flex-1"
          />
          <Button type="submit" variant="outline" size="sm" pending={pending}>
            {t("save")}
          </Button>
        </span>
      </Field>
      {state.error && <ErrorText>{state.error}</ErrorText>}
      {state.saved && !pending && (
        <p className="text-sm text-muted">{t("saved")}</p>
      )}
    </form>
  );
}
