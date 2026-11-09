"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Stack } from "@/shared/ui/carbon/stack";
import { ConfirmButton } from "@/shared/ui/confirm-button";
import { Notice } from "@/shared/ui/notice";
import { InviteLinkSnippet } from "./invite-link-snippet";

type Props = {
  tenant: string;
  /** Slug do torneio, no link de inscrição; sem ele, o convite da comunidade. */
  tournament?: string;
  /** O token de agora; volta do servidor quando alguém troca o link. */
  token: string;
  /** O nome do link, para o leitor de tela. */
  linkLabel: string;
  /** A consequência de trocar o link, numa frase, na confirmação. */
  rotateConfirm: string;
};

/**
 * Nas Configurações: o link de convite ou de inscrição à vista, o código
 * curto e "Gerar novo link", com a confirmação do que acontece. Trocado, o
 * link novo aparece com o foco no copiar.
 */
export function InviteLinkSettings({
  tenant,
  tournament,
  token,
  linkLabel,
  rotateConfirm,
}: Props) {
  const t = useTranslations("Invite");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  // O link novo vale até o refresh trazer o token de volta: se trouxer outro
  // (alguém trocou depois), o do servidor é o que vale.
  const [rotated, setRotated] = useState<{ from: string; to: string }>();
  const [justRotated, setJustRotated] = useState(false);
  const current = rotated?.from === token ? rotated.to : token;

  const rotate = () => {
    setJustRotated(false);
    run(
      () =>
        tournament
          ? api.POST("/tenants/{slug}/tournaments/{tournament}/invite/rotate", {
              params: { path: { slug: tenant, tournament } },
            })
          : api.POST("/tenants/{slug}/invite/rotate", {
              params: { path: { slug: tenant } },
            }),
      (link) => {
        setRotated({ from: token, to: link.token });
        setJustRotated(true);
        router.refresh();
      },
    );
  };

  return (
    <Stack gap={5}>
      {justRotated && <Notice kind="success">{t("rotated")}</Notice>}
      <InviteLinkSnippet token={current} label={linkLabel} disabled={pending} />
      <ActionWithNote error={error}>
        <ConfirmButton
          kind="tertiary"
          pending={pending}
          question={rotateConfirm}
          onConfirm={rotate}
        >
          {t("rotate")}
        </ConfirmButton>
      </ActionWithNote>
    </Stack>
  );
}
