"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { TOURNAMENT_TITLE_ID } from "../lib/focus";
import { TournamentReview } from "./tournament-review";

type Props = {
  tenant: string;
  /** O rascunho, como está salvo: a revisão sai dele. */
  tournament: components["schemas"]["Tournament"];
  /**
   * A data do rascunho ficou para trás (passou, ou ficou mais perto que a
   * janela de check-in): o botão fica desabilitado, com o motivo.
   */
  startBehind?: "past" | "window";
};

/**
 * Rascunho → inscrição. A confirmação mostra a revisão do que vai valer (a
 * configuração salva) e diz que publicado o torneio não volta a rascunho nem
 * pode ser excluído. A API confere os campeões e a data de novo.
 */
export function PublishTournamentButton({
  tenant,
  tournament,
  startBehind,
}: Props) {
  const t = useTranslations("Tournament");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <ActionWithNote
      error={error}
      note={startBehind && t(`publishBlocked.${startBehind}`)}
      compact
      blocked={startBehind !== undefined}
    >
      <ConfirmButton
        destructive={false}
        kind="primary"
        size="md"
        pending={pending}
        disabled={startBehind !== undefined}
        question={t("publishConfirm")}
        details={<TournamentReview tournament={tournament} />}
        onConfirm={() => {
          run(
            () =>
              api.POST("/tenants/{slug}/tournaments/{tournament}/publish", {
                params: {
                  path: { slug: tenant, tournament: tournament.slug },
                },
              }),
            () => {
              router.refresh();
              focusIfLost(TOURNAMENT_TITLE_ID);
            },
          );
        }}
      >
        {t("publish")}
      </ConfirmButton>
    </ActionWithNote>
  );
}
