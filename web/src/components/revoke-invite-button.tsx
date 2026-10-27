"use client";

import { useTranslations } from "next-intl";
import { useState, useTransition } from "react";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/lib/api/client";
import { useApiErrorMessage } from "@/lib/api/use-api-error";

export function RevokeInviteButton({
  slug,
  inviteId,
}: {
  slug: string;
  inviteId: number;
}) {
  const t = useTranslations("Tenant.invites");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const [pending, startTransition] = useTransition();
  const [error, setError] = useState<string>();

  return (
    <span className="flex flex-col items-end">
      <button
        type="button"
        disabled={pending}
        onClick={() =>
          startTransition(async () => {
            const result = await api
              .DELETE("/tenants/{slug}/invites/{invite_id}", {
                params: { path: { slug, invite_id: inviteId } },
              })
              .catch(() => undefined);
            if (result?.response.ok) {
              router.refresh();
            } else {
              setError(errorMessage(result?.error?.error));
            }
          })
        }
        className="text-sm text-red-600 underline-offset-4 hover:underline disabled:opacity-50 dark:text-red-400"
      >
        {t("revoke")}
      </button>
      {error && (
        <span role="alert" className="text-xs text-red-600 dark:text-red-400">
          {error}
        </span>
      )}
    </span>
  );
}
