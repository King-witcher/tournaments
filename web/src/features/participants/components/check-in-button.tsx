"use client";

import { useTranslations } from "next-intl";
import { TOURNAMENT_TITLE_ID } from "@/features/tournaments/lib/focus";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button, type ButtonKind } from "@/shared/ui/button";
import { MY_PARTICIPATION_ID } from "../lib/focus";

type Props = {
  tenant: string;
  slug: string;
  /** `primary` para quem joga; `tertiary` para a organização. */
  kind?: ButtonKind;
};

/**
 * O check-in de quem vê, durante o check-in. Desfazer fica no ⋯ do
 * cabeçalho. Feito, o botão some: o foco vai para "Sua situação".
 */
export function CheckInButton({ tenant, slug, kind = "primary" }: Props) {
  const t = useTranslations("Participants.checkIn");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <ActionWithNote error={error}>
      <Button
        kind={kind}
        pending={pending}
        onClick={() =>
          run(
            () =>
              api.POST("/tenants/{slug}/tournaments/{tournament}/check-in", {
                params: { path: { slug: tenant, tournament: slug } },
              }),
            () => {
              router.refresh();
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
