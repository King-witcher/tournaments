"use client";

import { useTranslations } from "next-intl";

import { ActionWithNote } from "@/features/tournaments/components/action-with-note";
import { TOURNAMENT_INVITES_ID } from "@/features/tournaments/lib/focus";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { focusIfLost } from "@/shared/lib/focus";
import { ConfirmButton } from "@/shared/ui/confirm-button";

type Props = { tenant: string; inviteId: number };

/**
 * Revogar um link de inscrição, na linha dele (pede confirmação). É a mesma
 * rota do convite da comunidade. Revogado, a linha some e o foco vai para a
 * lista dos links.
 */
export function TournamentInviteActions({ tenant, inviteId }: Props) {
  const t = useTranslations("Tournament.invites");
  const router = useRouter();
  const { run, pending, error } = useApiAction();

  return (
    <ActionWithNote align="end" error={error}>
      <ConfirmButton
        kind="danger--ghost"
        pending={pending}
        question={t("revokeConfirm")}
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
