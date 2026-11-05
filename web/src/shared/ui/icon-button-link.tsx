"use client";

import type { ReactNode } from "react";

import { type Href, Link as IntlLink } from "@/i18n/navigation";
import type { ButtonSize } from "./button";
import { Button as CarbonButton } from "./carbon/button";

type Props = {
  /** Destino, com o prefixo de idioma posto pelo `Link` do i18n. */
  href: Href;
  /** O ícone do `@carbon/icons-react` (`<Launch />`). */
  children: ReactNode;
  /** Para onde leva: vira o tooltip e o nome acessível. Obrigatório, porque sem texto o link não tem nome. */
  label: string;
  /** `ghost` por padrão: ação de apoio ao lado de um texto ou numa linha. */
  kind?: "primary" | "secondary" | "tertiary" | "ghost";
  /** `md` por padrão; `sm` em linha de tabela ou de lista. */
  size?: ButtonSize;
  /** Abre numa guia nova, sem pré-carregar a página (a outra guia não aproveitaria). */
  newTab?: boolean;
  className?: string;
};

/**
 * Link só com ícone, com cara de `IconButton` (Carbon `Button` `hasIconOnly`
 * sobre o `Link` do i18n): o `label` é o tooltip e o nome acessível. Para ir
 * a uma página ao lado de um texto que já a nomeia ("Abrir em nova guia" ao
 * lado do link copiável). Link com texto: `ButtonLink` ou `Link`.
 */
export function IconButtonLink({
  href,
  children,
  label,
  kind = "ghost",
  size = "md",
  newTab,
  className,
}: Props) {
  return (
    <CarbonButton
      as={IntlLink}
      // O Carbon tipa href como string; quem o recebe é o Link do i18n.
      href={href as string}
      hasIconOnly
      iconDescription={label}
      // A dica muda de lado quando sairia da tela (no celular, o link fica
      // encostado na margem). O Carbon só tipa no IconButton, mas o Button o
      // repassa a ele.
      {...{ autoAlign: true }}
      kind={kind}
      size={size}
      className={className}
      {...(newTab && { target: "_blank", rel: "noopener", prefetch: false })}
    >
      {children}
    </CarbonButton>
  );
}
