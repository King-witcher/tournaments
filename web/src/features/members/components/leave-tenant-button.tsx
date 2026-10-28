"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { ConfirmButton } from "@/shared/ui/confirm-button";

/** Sai da comunidade (pede confirmação) e volta para a home. */
export function LeaveTenantButton({ slug }: { slug: string }) {
  const t = useTranslations("Tenant.leave");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <div className="flex flex-col items-start gap-1">
      <ConfirmButton
        variant="outline"
        size="sm"
        pending={pending}
        className="text-red-600 dark:text-red-400"
        question={t("confirm")}
        onConfirm={() => {
          run(
            () =>
              api.DELETE("/tenants/{slug}/me", {
                params: { path: { slug } },
              }),
            () => {
              router.push("/");
              router.refresh();
            },
          );
        }}
      >
        {t("submit")}
      </ConfirmButton>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
