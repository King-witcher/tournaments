"use client";

import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useRef, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ActionWithNote } from "@/shared/ui/action-with-note";
import { Button, type ButtonKind, type ButtonSize } from "@/shared/ui/button";
import { ConfirmModal } from "@/shared/ui/confirm-modal";
import { Notice } from "@/shared/ui/notice";
import { PassiveModal } from "@/shared/ui/passive-modal";
import { COPY_BUTTON, InviteLinkSnippet } from "./invite-link-snippet";

type Props = {
  tenant: string;
  /** Slug do torneio, no link de inscrição; sem ele, o convite da comunidade. */
  tournament?: string;
  /** O token de agora; volta do servidor quando alguém troca o link. */
  token: string;
  /** O nome do botão. */
  label: string;
  /** O título do Modal. */
  title: string;
  /** O que o link faz, numa frase. */
  note: string;
  /** O nome do link, para o leitor de tela. */
  linkLabel: string;
  /** Por que o link não inscreve agora (as vagas acabaram). */
  warning?: string;
  /** A consequência de trocar o link, numa frase, na confirmação. */
  rotateConfirm: string;
  kind?: ButtonKind;
  size?: ButtonSize;
};

/**
 * O botão que abre o link de convite ou de inscrição, com o botão de copiar
 * e, como ação secundária, "Gerar novo link". A confirmação toma o lugar do
 * Modal do link, e não fica por cima dele: o Carbon não empilha Modais (um
 * clique no de cima contaria como clique fora do de baixo). Depois de trocar,
 * o link volta a aparecer, já o novo, com o foco em copiar. Ao fechar, o foco
 * volta ao botão.
 */
export function InviteLinkButton({
  tenant,
  tournament,
  token,
  label,
  title,
  note,
  linkLabel,
  warning,
  rotateConfirm,
  kind,
  size,
}: Props) {
  const t = useTranslations("Invite");
  const router = useRouter();
  const trigger = useRef<HTMLButtonElement>(null);
  const { run, pending, error } = useApiAction();
  const [view, setView] = useState<"closed" | "link" | "confirm">("closed");
  // O link novo vale até o refresh trazer o token de volta: se trouxer outro
  // (alguém trocou depois), o do servidor é o que vale.
  const [rotated, setRotated] = useState<{ from: string; to: string }>();
  const [justRotated, setJustRotated] = useState(false);
  const current = rotated?.from === token ? rotated.to : token;

  const rotate = () => {
    setView("link");
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
    <>
      <Button
        ref={trigger}
        kind={kind}
        size={size}
        onClick={() => setView("link")}
      >
        {label}
      </Button>
      {view === "link" && (
        <PassiveModal
          title={title}
          returnFocusTo={trigger}
          selectorPrimaryFocus={COPY_BUTTON}
          onClose={() => {
            setView("closed");
            setJustRotated(false);
          }}
        >
          <Stack gap={5}>
            {warning && <Notice kind="warning">{warning}</Notice>}
            {justRotated && <Notice kind="success">{t("rotated")}</Notice>}
            <p>{note}</p>
            <InviteLinkSnippet
              token={current}
              label={linkLabel}
              disabled={pending}
            />
            <ActionWithNote error={error}>
              <Button
                kind="ghost"
                size="sm"
                pending={pending}
                onClick={() => setView("confirm")}
              >
                {t("rotate")}
              </Button>
            </ActionWithNote>
          </Stack>
        </PassiveModal>
      )}
      <ConfirmModal
        open={view === "confirm"}
        title={t("rotate")}
        confirmLabel={t("rotate")}
        destructive
        onClose={() => setView("link")}
        onConfirm={rotate}
      >
        {rotateConfirm}
      </ConfirmModal>
    </>
  );
}
