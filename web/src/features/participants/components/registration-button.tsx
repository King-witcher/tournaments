"use client";

import { useTranslations } from "next-intl";

import { ActionWithNote } from "@/features/tournaments/components/action-with-note";
import { TOURNAMENT_TITLE_ID } from "@/features/tournaments/lib/focus";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { Button, type ButtonKind } from "@/shared/ui/button";
import { MY_PARTICIPATION_ID } from "../lib/focus";

type Props = {
  tenant: string;
  slug: string;
  /** `primary` para quem joga; `tertiary` para a organização (o primário dela é o ciclo). */
  kind?: ButtonKind;
};

/**
 * Inscreve quem vê no torneio. Cancelar a inscrição fica no ⋯ do cabeçalho,
 * com confirmação. Inscrito, o botão some: o foco vai para "Sua situação",
 * que passa a dizer que ele está inscrito.
 */
export function RegistrationButton({ tenant, slug, kind = "primary" }: Props) {
  const t = useTranslations("Participants.registration");
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
              api.POST(
                "/tenants/{slug}/tournaments/{tournament}/registration",
                { params: { path: { slug: tenant, tournament: slug } } },
              ),
            () => {
              router.refresh();
              focusIfLost(MY_PARTICIPATION_ID);
              focusIfLost(TOURNAMENT_TITLE_ID);
            },
          )
        }
      >
        {t("register")}
      </Button>
    </ActionWithNote>
  );
}
