"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { ConfirmButton } from "@/shared/ui/confirm-button";

/**
 * Rascunho → inscrição, com confirmação: publicado, o torneio não volta a
 * rascunho nem pode ser excluído. A API confere os campeões de novo (e exige a
 * data de início).
 */
export function PublishTournamentButton({
  tenant,
  slug,
  disabled = false,
  skipsRegistration = false,
}: {
  tenant: string;
  slug: string;
  disabled?: boolean;
  /** O check-in já devia ter aberto: publicar pula as inscrições. */
  skipsRegistration?: boolean;
}) {
  const t = useTranslations("Tournament");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <div className="flex flex-col gap-1">
      <ConfirmButton
        danger={false}
        size="sm"
        pending={pending}
        disabled={disabled}
        question={t(
          skipsRegistration ? "publishConfirmCheckIn" : "publishConfirm",
        )}
        onConfirm={() => {
          run(
            () =>
              api.POST("/tenants/{slug}/tournaments/{tournament}/publish", {
                params: { path: { slug: tenant, tournament: slug } },
              }),
            () => router.refresh(),
          );
        }}
      >
        {t(skipsRegistration ? "publishToCheckIn" : "publish")}
      </ConfirmButton>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
