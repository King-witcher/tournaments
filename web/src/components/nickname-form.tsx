"use client";

import { useTranslations } from "next-intl";
import { useActionState, useState } from "react";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/lib/api/client";
import { useApiErrorMessage } from "@/lib/api/use-api-error";

type State = { saved: boolean; error?: string };

/** Apelido do usuário logado neste tenant. */
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

  const [state, submit, pending] = useActionState<State>(
    async () => {
      const result = await api
        .PATCH("/tenants/{slug}/me", {
          params: { path: { slug } },
          body: { nickname: value },
        })
        .catch(() => undefined);
      if (result?.data) {
        setValue(result.data.nickname);
        router.refresh();
        return { saved: true };
      }
      return { saved: false, error: errorMessage(result?.error?.error) };
    },
    { saved: false },
  );

  return (
    <form action={submit} className="flex flex-col gap-1">
      <label className="flex flex-col gap-1">
        <span className="text-sm font-medium">{t("label")}</span>
        <span className="flex gap-2">
          <input
            required
            maxLength={32}
            value={value}
            onChange={(e) => setValue(e.target.value)}
            className="flex-1 rounded-md border border-black/20 bg-transparent px-3 py-2 dark:border-white/20"
          />
          <button
            type="submit"
            disabled={pending}
            className="rounded-md border border-black/15 px-3 py-2 text-sm hover:bg-black/5 disabled:opacity-50 dark:border-white/20 dark:hover:bg-white/10"
          >
            {t("save")}
          </button>
        </span>
      </label>
      {state.error && (
        <p role="alert" className="text-sm text-red-600 dark:text-red-400">
          {state.error}
        </p>
      )}
      {state.saved && !pending && (
        <p className="text-sm opacity-70">{t("saved")}</p>
      )}
    </form>
  );
}
