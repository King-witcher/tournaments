"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { ConfirmButton } from "@/shared/ui/confirm-button";

/** Exclui o rascunho (pede confirmação) e volta para o tenant. */
export function DeleteTournamentButton({
  tenant,
  slug,
}: {
  tenant: string;
  slug: string;
}) {
  const t = useTranslations("Tournament");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <div className="flex flex-col gap-1">
      <ConfirmButton
        variant="outline"
        size="sm"
        pending={pending}
        question={t("deleteConfirm")}
        onConfirm={() => {
          run(
            () =>
              api.DELETE("/tenants/{slug}/tournaments/{tournament}", {
                params: { path: { slug: tenant, tournament: slug } },
              }),
            () => {
              router.push(`/${tenant}`);
              router.refresh();
            },
          );
        }}
      >
        {t("delete")}
      </ConfirmButton>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
