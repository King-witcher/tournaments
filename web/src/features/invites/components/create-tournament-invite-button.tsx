"use client";

import { useTranslations } from "next-intl";
import { useRef, useState } from "react";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button } from "@/shared/ui/button";
import { PassiveModal } from "@/shared/ui/passive-modal";
import { InviteLink } from "./invite-link";

type Props = { tenant: string; tournament: string };

/**
 * Gera um link de inscrição no torneio e o mostra uma vez, num Modal, para
 * copiar (a API só guarda o hash). É o mesmo `/invite/{token}` do convite da
 * comunidade. Ao fechar, o foco volta ao botão.
 */
export function CreateTournamentInviteButton({ tenant, tournament }: Props) {
  const t = useTranslations("Tournament.invites");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const trigger = useRef<HTMLButtonElement>(null);
  const [link, setLink] = useState<string>();

  return (
    <ActionWithNote error={error}>
      <Button
        ref={trigger}
        kind="tertiary"
        size="sm"
        pending={pending}
        onClick={() =>
          run(
            () =>
              api.POST("/tenants/{slug}/tournaments/{tournament}/invites", {
                params: { path: { slug: tenant, tournament } },
              }),
            (invite) => {
              setLink(`${window.location.origin}/invite/${invite.token}`);
              router.refresh();
            },
          )
        }
      >
        {t("create")}
      </Button>
      {link && (
        <PassiveModal
          title={t("createdTitle")}
          returnFocusTo={trigger}
          onClose={() => setLink(undefined)}
        >
          <InviteLink link={link} />
        </PassiveModal>
      )}
    </ActionWithNote>
  );
}
