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

/**
 * Remove um login da conta, confirmando a senha atual: o nome de usuário e a
 * senha (`password`) ou o Discord (`discord`).
 */
export function RemoveLoginForm({ login }: { login: "password" | "discord" }) {
  const t = useTranslations("Account");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const [current, setCurrent] = useState("");

  const [error, submit, pending] = useActionState(async () => {
    const body = { current_password: current };
    const result =
      login === "password"
        ? await callApi(api.DELETE("/auth/password", { body }))
        : await callApi(api.DELETE("/auth/discord", { body }));
    if (!result.ok) return errorMessage(result.code, result.params);
    setCurrent("");
    router.refresh();
    return undefined;
  }, undefined);

  return (
    <form action={submit} className="flex flex-col gap-1">
      <Field label={t("currentPassword")}>
        <span className="flex gap-2">
          <Input
            required
            type="password"
            autoComplete="current-password"
            value={current}
            onChange={(e) => setCurrent(e.target.value)}
            className="flex-1"
          />
          <Button type="submit" variant="outline" size="sm" disabled={pending}>
            {login === "password" ? t("password.remove") : t("discord.unlink")}
          </Button>
        </span>
      </Field>
      {error && <ErrorText>{error}</ErrorText>}
    </form>
  );
}
