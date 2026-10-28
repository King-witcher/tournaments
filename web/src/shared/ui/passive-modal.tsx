"use client";

import { Modal } from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ReactNode, RefObject } from "react";
import { createPortal } from "react-dom";

import { useReturnFocus } from "@/shared/hooks/use-return-focus";

type Props = {
  title: ReactNode;
  /** A informação (um link gerado para copiar). */
  children: ReactNode;
  /** Fechou (X, Esc ou clique fora). O foco volta ao botão que abriu. */
  onClose: () => void;
  /** O botão que abriu, se o navegador não der foco a ele no clique; sem isso, quem tinha o foco. */
  returnFocusTo?: RefObject<HTMLElement | null>;
};

/**
 * Modal só de informação, sem ação a confirmar: título, conteúdo e o X para
 * fechar (ex.: o link gerado, que a API só mostra uma vez). Ao fechar, o foco
 * volta ao botão que abriu. Existe só enquanto aberto, fora da árvore (no
 * `body`): monte com `{link && <PassiveModal …>}`.
 */
export function PassiveModal(props: Props) {
  return createPortal(<OpenPassiveModal {...props} />, document.body);
}

function OpenPassiveModal({ title, children, onClose, returnFocusTo }: Props) {
  const t = useTranslations("Common");
  const returnFocus = useReturnFocus(returnFocusTo);

  return (
    <Modal
      open
      passiveModal
      size="sm"
      modalHeading={title}
      closeButtonLabel={t("close")}
      onRequestClose={() => {
        onClose();
        returnFocus();
      }}
    >
      {children}
    </Modal>
  );
}
