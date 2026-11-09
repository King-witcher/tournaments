"use client";

import { useTranslations } from "next-intl";
import { useEffect, useRef } from "react";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { CodeSnippet } from "@/shared/ui/code-snippet";
import { isShortCode } from "../lib/invite-input";
import styles from "./invite-link-snippet.module.scss";

type Props = {
  token: string;
  /** O nome do link, para o leitor de tela. */
  label: string;
  /** O link está sendo trocado: este não vale mais. */
  disabled: boolean;
};

/**
 * O link, com o botão de copiar do Carbon, e o código para digitar no
 * aplicativo, quando o convite é do formato curto. Vai sem idioma: o proxy do
 * next-intl manda cada pessoa para o seu. O endereço do site só existe no
 * navegador, então a primeira pintura mostra o caminho. Terminada a troca, o
 * foco volta para o copiar, que não recebe foco enquanto está desabilitado.
 */
export function InviteLinkSnippet({ token, label, disabled }: Props) {
  const t = useTranslations("Invite");
  const wrapper = useRef<HTMLDivElement>(null);
  const wasDisabled = useRef(disabled);
  const isClient = useIsClient();
  const origin = isClient ? window.location.origin : "";

  useEffect(() => {
    if (wasDisabled.current && !disabled) {
      wrapper.current?.querySelector("button")?.focus();
    }
    wasDisabled.current = disabled;
  }, [disabled]);

  return (
    <>
      <div ref={wrapper} className={styles.link}>
        <CodeSnippet
          type="single"
          align="bottom-end"
          aria-label={label}
          disabled={disabled}
        >
          {`${origin}/invite/${token}`}
        </CodeSnippet>
      </div>
      {isShortCode(token) && (
        <div className={styles.code}>
          <span className={styles.label}>{t("code")}</span>
          <CodeSnippet
            type="inline"
            aria-label={t("copyCode")}
            disabled={disabled}
          >
            {token}
          </CodeSnippet>
        </div>
      )}
    </>
  );
}
