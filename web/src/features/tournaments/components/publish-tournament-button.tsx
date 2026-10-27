"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";

/** Rascunho → inscrição. A API confere os campeões de novo. */
export function PublishTournamentButton({
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
        size="sm"
        disabled={pending}
        onClick={() =>
          run(
            () =>
              api.POST("/tenants/{slug}/tournaments/{tournament}/publish", {
                params: { path: { slug: tenant, tournament: slug } },
              }),
            () => router.refresh(),
          )
        }
      >
        {t("publish")}
      </Button>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
