"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";

type Props = {
  tenant: string;
  tournament: string;
  locale: components["schemas"]["Locale"];
};

/** Remove a tradução: o idioma volta a mostrar o original. */
export function DeleteTranslationButton({ tenant, tournament, locale }: Props) {
  const t = useTranslations("Tournament.translation");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <div className="flex flex-col gap-1">
      <Button
        variant="link"
        className="py-2"
        disabled={pending}
        onClick={() =>
          run(
            () =>
              api.DELETE(
                "/tenants/{slug}/tournaments/{tournament}/translations/{locale}",
                { params: { path: { slug: tenant, tournament, locale } } },
              ),
            () => router.refresh(),
          )
        }
      >
        {t("remove")}
      </Button>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
