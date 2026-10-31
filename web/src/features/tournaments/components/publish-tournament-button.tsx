"use client";

import { useTranslations } from "next-intl";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import type { ButtonKind } from "@/shared/ui/button";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { TOURNAMENT_TITLE_ID } from "../lib/focus";

type Props = {
  tenant: string;
  slug: string;
  /**
   * `primary` no cabeçalho, onde o motivo vira a dica do botão; `secondary`
   * na aba Organização, onde ele fica escrito embaixo.
   */
  kind?: ButtonKind;
  /**
   * A data do rascunho ficou para trás (passou, ou ficou mais perto que a
   * janela de check-in): o botão fica desabilitado, com o motivo.
   */
  startBehind?: "past" | "window";
};

/**
 * Rascunho → inscrição, com confirmação: publicado, o torneio não volta a
 * rascunho nem pode ser excluído. A API confere os campeões e a data de novo.
 */
export function PublishTournamentButton({
  tenant,
  slug,
  kind = "primary",
  startBehind,
}: Props) {
  const t = useTranslations("Tournament");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <ActionWithNote
      error={error}
      note={startBehind && t(`publishBlocked.${startBehind}`)}
      compact={kind === "primary"}
      blocked={startBehind !== undefined}
    >
      <ConfirmButton
        destructive={false}
        kind={kind}
        size="md"
        pending={pending}
        disabled={startBehind !== undefined}
        question={t("publishConfirm")}
        onConfirm={() => {
          run(
            () =>
              api.POST("/tenants/{slug}/tournaments/{tournament}/publish", {
                params: { path: { slug: tenant, tournament: slug } },
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
