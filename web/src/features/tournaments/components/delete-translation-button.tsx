"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ErrorText } from "@/shared/ui/alert";
import { ConfirmButton } from "@/shared/ui/confirm-button";

type Props = {
  tenant: string;
  tournament: string;
  locale: components["schemas"]["Locale"];
  /** `id` do campo que recebe o foco quando o botão some (o nome da tradução). */
  focusAfter: string;
  onRemoved: () => void;
};

/** Remove a tradução na hora: o idioma volta a mostrar o original. */
export function DeleteTranslationButton({
  tenant,
  tournament,
  locale,
  focusAfter,
  onRemoved,
}: Props) {
  const t = useTranslations("TournamentForm.translation");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <div>
      <ConfirmButton
        size="md"
        pending={pending}
        question={t("removeConfirm")}
        onConfirm={() =>
          run(
            () =>
              api.DELETE(
                "/tenants/{slug}/tournaments/{tournament}/translations/{locale}",
                { params: { path: { slug: tenant, tournament, locale } } },
              ),
            () => {
              onRemoved();
              router.refresh();
              // O botão some com a tradução: o foco vai para o nome dela.
              focusIfLost(focusAfter);
            },
          )
        }
      >
        {t("remove")}
      </ConfirmButton>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
