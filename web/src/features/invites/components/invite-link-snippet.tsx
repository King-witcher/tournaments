"use client";

import { useEffect, useRef } from "react";

import { CodeSnippet } from "@/shared/ui/code-snippet";
import styles from "./invite-link-snippet.module.scss";

type Props = {
  token: string;
  /** O nome do link, para o leitor de tela. */
  label: string;
  /** O link está sendo trocado: este não vale mais. */
  disabled: boolean;
};

/** O botão de copiar, que o Modal foca ao abrir: o próximo passo é copiar. */
export const COPY_BUTTON = `.${styles.link} button:not(:disabled)`;

/**
 * O link, com o botão de copiar do Carbon. Vai sem idioma: o proxy do next-intl
 * manda cada pessoa para o seu. Só existe dentro do Modal, no navegador, então
 * já sabe o endereço do site. Terminada a troca, o foco volta para o copiar,
 * que não recebe foco enquanto está desabilitado.
 */
export function InviteLinkSnippet({ token, label, disabled }: Props) {
  const wrapper = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!disabled) wrapper.current?.querySelector("button")?.focus();
  }, [disabled]);

  return (
    <div ref={wrapper} className={styles.link}>
      <CodeSnippet type="single" aria-label={label} disabled={disabled}>
        {`${window.location.origin}/invite/${token}`}
      </CodeSnippet>
    </div>
  );
}
