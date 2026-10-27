"use client";

import { useTranslations } from "next-intl";
import { useState, useTransition } from "react";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/lib/api/client";
import { useApiErrorMessage } from "@/lib/api/use-api-error";

/** Aceita o convite e leva para a página da comunidade. */
export function AcceptInviteButton({ token }: { token: string }) {
  const t = useTranslations("Invite");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const [pending, startTransition] = useTransition();
  const [error, setError] = useState<string>();

  return (
    <div className="flex flex-col items-start gap-2">
      <button
        type="button"
        disabled={pending}
        onClick={() =>
          startTransition(async () => {
            const result = await api
              .POST("/invites/{token}/accept", {
                params: { path: { token } },
              })
              .catch(() => undefined);
            if (result?.data) {
              router.push(`/${result.data.slug}`);
            } else {
              setError(errorMessage(result?.error?.error));
            }
          })
        }
        className="rounded-md bg-black px-4 py-2 text-sm font-medium text-white hover:bg-black/80 disabled:opacity-50 dark:bg-white dark:text-black dark:hover:bg-white/80"
      >
        {t("accept")}
      </button>
      {error && (
        <p role="alert" className="text-sm text-red-600 dark:text-red-400">
          {error}
        </p>
      )}
    </div>
  );
}
