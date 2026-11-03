"use client";

import { useTranslations } from "next-intl";
import { TOURNAMENT_TITLE_ID } from "@/features/tournaments/lib/focus";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button, type ButtonKind, type ButtonSize } from "@/shared/ui/button";
import { MY_PARTICIPATION_ID } from "../lib/focus";

type Props = {
  tenant: string;
  slug: string;
  /** `primary` para quem joga; `tertiary` para a organização e nas listas. */
  kind?: ButtonKind;
  size?: ButtonSize;
  /**
   * Fora da página do torneio: o alvo do foco quando o botão some. Na página
   * dele, o foco vai para "Sua situação" e, sem ela, para o título.
   */
  focusAfter?: string;
};

/**
 * O check-in de quem vê, durante o check-in. Desfazer fica no ⋯ do
 * cabeçalho do torneio. Feito, o botão some, e o foco vai para o alvo.
 */
export function CheckInButton({
  tenant,
  slug,
  kind = "primary",
  size,
  focusAfter,
}: Props) {
  const t = useTranslations("Participants.checkIn");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <ActionWithNote error={error}>
      <Button
        kind={kind}
        size={size}
        pending={pending}
        onClick={() =>
          run(
            () =>
              api.POST("/tenants/{slug}/tournaments/{tournament}/check-in", {
                params: { path: { slug: tenant, tournament: slug } },
              }),
            () => {
              router.refresh();
              if (focusAfter) {
                focusIfLost(focusAfter);
                return;
              }
              focusIfLost(MY_PARTICIPATION_ID);
              focusIfLost(TOURNAMENT_TITLE_ID);
            },
          )
        }
      >
        {t("do")}
      </Button>
    </ActionWithNote>
  );
}
