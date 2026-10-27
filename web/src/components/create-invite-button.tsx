"use client";

import { useTranslations } from "next-intl";
import { useState, useTransition } from "react";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/lib/api/client";
import { useApiErrorMessage } from "@/lib/api/use-api-error";

/**
 * Gera um link de convite e mostra uma vez (a API só guarda o hash). O link
 * vai sem idioma: o proxy do next-intl manda cada pessoa para o seu.
 */
export function CreateInviteButton({ slug }: { slug: string }) {
  const t = useTranslations("Tenant.invites");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const [pending, startTransition] = useTransition();
  const [link, setLink] = useState<string>();
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState<string>();

  return (
    <div className="flex flex-col items-start gap-2">
      <button
        type="button"
        disabled={pending}
        onClick={() =>
          startTransition(async () => {
            const result = await api
              .POST("/tenants/{slug}/invites", { params: { path: { slug } } })
              .catch(() => undefined);
            if (result?.data) {
              setLink(`${window.location.origin}/invite/${result.data.token}`);
              setCopied(false);
              setError(undefined);
              router.refresh();
            } else {
              setError(errorMessage(result?.error?.error));
            }
          })
        }
        className="rounded-md bg-black px-4 py-2 text-sm font-medium text-white hover:bg-black/80 disabled:opacity-50 dark:bg-white dark:text-black dark:hover:bg-white/80"
      >
        {t("create")}
      </button>
      {error && (
        <p role="alert" className="text-sm text-red-600 dark:text-red-400">
          {error}
        </p>
      )}
      {link && (
        <div className="flex w-full flex-col gap-1 rounded-md border border-black/15 p-3 dark:border-white/20">
          <span className="flex gap-2">
            <input
              readOnly
              value={link}
              aria-label={t("createdHint")}
              onFocus={(e) => e.target.select()}
              className="flex-1 rounded-md border border-black/20 bg-transparent px-3 py-2 font-mono text-sm dark:border-white/20"
            />
            <button
              type="button"
              onClick={() =>
                navigator.clipboard.writeText(link).then(() => setCopied(true))
              }
              className="rounded-md border border-black/15 px-3 py-2 text-sm hover:bg-black/5 dark:border-white/20 dark:hover:bg-white/10"
            >
              {copied ? t("copied") : t("copy")}
            </button>
          </span>
          <span className="text-xs opacity-70">{t("createdHint")}</span>
        </div>
      )}
    </div>
  );
}
