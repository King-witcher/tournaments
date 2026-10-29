"use client";

import { useTranslations } from "next-intl";
import { TOURNAMENT_INVITES_ID } from "@/features/tournaments/lib/focus";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { ConfirmButton } from "@/shared/ui/confirm-button";

type Props = {
  tenant: string;
  inviteId: number;
  /** O começo do link (`…/invite/7Fq2…`), para a confirmação dizer qual; nulo nos antigos. */
  hint: string | null;
};

/**
 * Revogar um link de inscrição, na linha dele (pede confirmação). É a mesma
 * rota do convite da comunidade. Revogado, a linha some e o foco vai para a
 * lista dos links.
 */
export function TournamentInviteActions({ tenant, inviteId, hint }: Props) {
  const t = useTranslations("Tournament.invites");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <ActionWithNote align="end" error={error}>
      <ConfirmButton
        kind="danger--ghost"
        pending={pending}
        question={t("revokeConfirm", {
          has: hint ? "yes" : "no",
          link: hint ?? "",
        })}
        onConfirm={() =>
          run(
            () =>
              api.DELETE("/tenants/{slug}/invites/{invite_id}", {
                params: { path: { slug: tenant, invite_id: inviteId } },
              }),
            () => {
              router.refresh();
              focusIfLost(TOURNAMENT_INVITES_ID);
            },
          )
        }
      >
        {t("revoke")}
      </ConfirmButton>
    </ActionWithNote>
  );
}
