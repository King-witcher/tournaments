"use client";

import { useTranslations } from "next-intl";
import type { ReactNode, Ref, RefObject } from "react";
import { createPortal } from "react-dom";

import { useReturnFocus } from "@/shared/hooks/use-return-focus";
import { Button } from "./button";
import {
  ComposedModal,
  ModalBody,
  ModalFooter,
  ModalHeader,
} from "./carbon/modal";
import { usePrefix } from "./carbon/prefix";
import { Stack } from "./carbon/stack";
import { FormError } from "./form-error";

type Props = {
  /** Título curto (o nome da ação). */
  title: ReactNode;
  /**
   * Os campos (`TextInput`, `Select`…); o primeiro recebe o foco ao abrir,
   * ou o primeiro elemento com `data-modal-primary-focus` que vier antes
   * dele (um controle que não é campo, como a área do editor de foto).
   */
  children: ReactNode;
  /** Texto do primário, com o nome da ação ("Inscrever"). */
  submitLabel: ReactNode;
  /** Enviou: o primário ou o Enter num campo. Não roda com `pending` nem `submitDisabled`. */
  onSubmit: () => void;
  /** Fechou sem enviar (Cancelar, Esc ou X). O foco volta ao botão que abriu. */
  onClose: () => void;
  /** A ação está em andamento: o primário mostra o indicador e ignora o clique. */
  pending?: boolean;
  /** Mantém o primário travado (falta preencher algo). */
  submitDisabled?: boolean;
  /** Primário destrutivo (`danger`), ex.: "Remover senha". */
  destructive?: boolean;
  /** O erro geral (o que a API devolveu, já traduzido), acima dos botões. */
  error?: ReactNode;
  /** O botão que abriu, se o navegador não der foco a ele no clique; sem isso, quem tinha o foco. */
  returnFocusTo?: RefObject<HTMLElement | null>;
  /** O `<form>` de dentro, para o `useFormValidity` de quem abre. */
  formRef?: Ref<HTMLFormElement>;
};

/**
 * Modal com um formulário curto (um ou dois campos): título, campos, erro
 * geral, "Cancelar" e o primário com o nome da ação, que fica `pending`. O
 * foco começa no primeiro campo, Enter envia e, ao fechar sem enviar, o foco
 * volta ao botão que abriu. Existe só enquanto aberto, fora da árvore (no
 * `body`): monte com `{open && <FormModal …>}`, de dentro de um componente
 * que guarda o estado dos campos e só existe aberto, para cada abertura
 * começar sem valor nem erro. Depois de enviar com sucesso, quem abriu
 * desmonta o modal e leva o foco para onde faz sentido.
 */
export function FormModal(props: Props) {
  return createPortal(<OpenFormModal {...props} />, document.body);
}

function OpenFormModal({
  title,
  children,
  submitLabel,
  onSubmit,
  onClose,
  pending,
  submitDisabled,
  destructive,
  error,
  returnFocusTo,
  formRef,
}: Props) {
  const t = useTranslations("Common");
  const prefix = usePrefix();
  const returnFocus = useReturnFocus(returnFocusTo);
  const content = `.${prefix}--modal-content`;

  const close = () => {
    onClose();
    returnFocus();
  };
  const submit = () => {
    if (pending || submitDisabled) return;
    onSubmit();
  };

  return (
    <ComposedModal
      open
      size="sm"
      onClose={close}
      selectorPrimaryFocus={`${content} [data-modal-primary-focus], ${content} input:not([type="hidden"]), ${content} select, ${content} textarea`}
    >
      <ModalHeader title={title} iconDescription={t("close")} />
      <ModalBody>
        <form
          ref={formRef}
          onSubmit={(event) => {
            event.preventDefault();
            submit();
          }}
        >
          <Stack gap={5}>
            {children}
            {error && <FormError>{error}</FormError>}
          </Stack>
          {/* Sem um botão de envio no formulário, o Enter num campo não envia. */}
          <button type="submit" hidden />
        </form>
      </ModalBody>
      <ModalFooter>
        <Button kind="secondary" onClick={close}>
          {t("cancel")}
        </Button>
        <Button
          kind={destructive ? "danger" : "primary"}
          pending={pending}
          disabled={submitDisabled}
          onClick={submit}
        >
          {submitLabel}
        </Button>
      </ModalFooter>
    </ComposedModal>
  );
}
