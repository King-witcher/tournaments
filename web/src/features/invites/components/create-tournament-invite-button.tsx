"use client";

import { Modal } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useRef, useState } from "react";
import { createPortal } from "react-dom";

import { ActionWithNote } from "@/features/tournaments/components/action-with-note";
import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { Button } from "@/shared/ui/button";
import { InviteLink } from "./invite-link";

type Props = { tenant: string; tournament: string };

/**
 * Gera um link de inscrição no torneio e o mostra uma vez, num Modal, para
 * copiar (a API só guarda o hash). É o mesmo `/invite/{token}` do convite da
 * comunidade. Ao fechar, o foco volta ao botão.
 */
export function CreateTournamentInviteButton({ tenant, tournament }: Props) {
  const t = useTranslations("Tournament.invites");
  const tCommon = useTranslations("Common");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const trigger = useRef<HTMLButtonElement>(null);
  const [link, setLink] = useState<string>();

  const close = () => {
    setLink(undefined);
    // O Modal sai da árvore e o foco cairia no body: volta para o botão.
    requestAnimationFrame(() => trigger.current?.focus());
  };

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
      {link &&
        createPortal(
          <Modal
            open
            passiveModal
            size="sm"
            modalHeading={t("createdTitle")}
            closeButtonLabel={tCommon("close")}
            onRequestClose={close}
          >
            <InviteLink link={link} />
          </Modal>,
          document.body,
        )}
    </ActionWithNote>
  );
}
