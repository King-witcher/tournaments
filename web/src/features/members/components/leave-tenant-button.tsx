"use client";

import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { ErrorText } from "@/shared/ui/error-text";

/** Sai da comunidade (pede confirmação) e volta para o início. */
export function LeaveTenantButton({ slug }: { slug: string }) {
  const t = useTranslations("Tenant.leave");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <Stack gap={3}>
      <div>
        <ConfirmButton
          size="md"
          pending={pending}
          question={t("confirm")}
          onConfirm={() =>
            run(
              () =>
                api.DELETE("/tenants/{slug}/me", {
                  params: { path: { slug } },
                }),
              () => {
                router.push("/");
                router.refresh();
              },
            )
          }
        >
          {t("submit")}
        </ConfirmButton>
      </div>
      {error && <ErrorText>{error}</ErrorText>}
    </Stack>
  );
}
