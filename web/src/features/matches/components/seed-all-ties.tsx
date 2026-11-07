"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { ConfirmButton } from "@/shared/ui/confirm-button";

/** Com mais de um empate a decidir: a ordem da chave em todos, e o torneio termina. */
export function SeedAllTies({
  tenant,
  slug,
}: {
  tenant: string;
  slug: string;
}) {
  const t = useTranslations("Bracket.ties");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const go = () =>
    run(
      () =>
        api.POST("/tenants/{slug}/tournaments/{tournament}/ties/seed", {
          params: { path: { slug: tenant, tournament: slug } },
        }),
      () => router.refresh(),
    );

  return (
    <ActionWithNote error={error}>
      <ConfirmButton
        destructive={false}
        kind="tertiary"
        pending={pending}
        question={`${t("seedAllQuestion")} ${t("ends")}`}
        onConfirm={go}
      >
        {t("seedAll")}
      </ConfirmButton>
    </ActionWithNote>
  );
}
