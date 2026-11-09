"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { TOURNAMENT_TITLE_ID } from "@/features/tournaments/lib/focus";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";

type Props = {
  tenant: string;
  slug: string;
  /** A fase a iniciar, de 2 em diante. */
  stage: number;
  /** Abrir o check-in: a fase pede check-in e ele ainda não abriu. */
  openCheckIn: boolean;
  /** Por que não dá para iniciar agora (já traduzido); sem ele, dá. */
  blocker?: string;
  /** Quantos entram e quem fica de fora: o que o Modal mostra antes de iniciar. */
  details: ReactNode;
};

/**
 * Para organizer e owner, o próximo passo da fase seguinte, um de cada vez e
 * como primário (como o próximo passo do torneio no cabeçalho): abrir o
 * check-in, se a fase pede e ele não abriu; depois, iniciar, com o Modal que
 * mostra quantos entram e quem fica de fora. O motivo de não dar para iniciar
 * fica embaixo do botão.
 * Depois de cada passo o botão some, e o foco vai para o título do torneio.
 */
export function StageOrganizerActions({
  tenant,
  slug,
  stage,
  openCheckIn,
  blocker,
  details,
}: Props) {
  const t = useTranslations("Stages");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const path = { slug: tenant, tournament: slug, stage };
  const done = () => {
    router.refresh();
    focusIfLost(TOURNAMENT_TITLE_ID);
  };

  if (openCheckIn) {
    return (
      <ActionWithNote error={error}>
        <Button
          pending={pending}
          onClick={() =>
            run(
              () =>
                api.POST(
                  "/tenants/{slug}/tournaments/{tournament}/stages/{stage}/check-in/open",
                  { params: { path } },
                ),
              done,
            )
          }
        >
          {t("openCheckIn", { number: stage })}
        </Button>
      </ActionWithNote>
    );
  }
  return (
    <ActionWithNote note={blocker} error={error}>
      <ConfirmButton
        destructive={false}
        kind="primary"
        size="md"
        pending={pending}
        disabled={!!blocker}
        title={t("start", { number: stage })}
        question={t("startQuestion", { number: stage })}
        details={details}
        onConfirm={() =>
          run(
            () =>
              api.POST(
                "/tenants/{slug}/tournaments/{tournament}/stages/{stage}/start",
                { params: { path } },
              ),
            done,
          )
        }
      >
        {t("start", { number: stage })}
      </ConfirmButton>
    </ActionWithNote>
  );
}
