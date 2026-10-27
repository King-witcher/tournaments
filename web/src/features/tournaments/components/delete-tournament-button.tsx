"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";

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
      <Button
        variant="outline"
        size="sm"
        disabled={pending}
        onClick={() => {
          if (!window.confirm(t("deleteConfirm"))) return;
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
      </Button>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
