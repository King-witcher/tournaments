"use client";

import { useTranslations } from "next-intl";
import { useState, useTransition } from "react";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/lib/api/client";
import type { components } from "@/lib/api/schema";
import { useApiErrorMessage } from "@/lib/api/use-api-error";

type Props = {
  slug: string;
  userId: number;
  role: components["schemas"]["AssignableRole"];
};

/** Para o owner: alterna o membro entre player e organizer. */
export function MemberRoleButton({ slug, userId, role }: Props) {
  const t = useTranslations("Tenant.members");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const [pending, startTransition] = useTransition();
  const [error, setError] = useState<string>();
  const next = role === "player" ? "organizer" : "player";

  return (
    <span className="flex flex-col items-end">
      <button
        type="button"
        disabled={pending}
        onClick={() =>
          startTransition(async () => {
            const result = await api
              .PUT("/tenants/{slug}/members/{user_id}/role", {
                params: { path: { slug, user_id: userId } },
                body: { role: next },
              })
              .catch(() => undefined);
            if (result?.data) {
              setError(undefined);
              router.refresh();
            } else {
              setError(errorMessage(result?.error?.error));
            }
          })
        }
        className="text-sm underline-offset-4 hover:underline disabled:opacity-50"
      >
        {next === "organizer" ? t("makeOrganizer") : t("makePlayer")}
      </button>
      {error && (
        <span role="alert" className="text-xs text-red-600 dark:text-red-400">
          {error}
        </span>
      )}
    </span>
  );
}
