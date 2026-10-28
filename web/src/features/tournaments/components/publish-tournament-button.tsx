"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";

/**
 * Rascunho → inscrição, com confirmação: publicado, o torneio não volta a
 * rascunho nem pode ser excluído. A API confere os campeões de novo (e exige a
 * data de início).
 */
export function PublishTournamentButton({
  tenant,
  slug,
  disabled = false,
}: {
  tenant: string;
  slug: string;
  disabled?: boolean;
}) {
  const t = useTranslations("Tournament");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <div className="flex flex-col gap-1">
      <Button
        size="sm"
        disabled={pending || disabled}
        onClick={() => {
          if (!window.confirm(t("publishConfirm"))) return;
          run(
            () =>
              api.POST("/tenants/{slug}/tournaments/{tournament}/publish", {
                params: { path: { slug: tenant, tournament: slug } },
              }),
            () => router.refresh(),
          );
        }}
      >
        {t("publish")}
      </Button>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
