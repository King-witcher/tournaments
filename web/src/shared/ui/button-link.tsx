"use client";

import type { CarbonIconType } from "@carbon/icons-react";
import { Button as CarbonButton } from "@carbon/react";
import type { ReactNode } from "react";

import { type Href, Link as IntlLink } from "@/i18n/navigation";
import type { ButtonKind, ButtonSize } from "./button";

type Common = {
  children: ReactNode;
  kind?: ButtonKind;
  size?: ButtonSize;
  renderIcon?: CarbonIconType;
  className?: string;
};

type Props = Common &
  (
    | {
        /** Destino, com o prefixo de idioma posto pelo `Link` do i18n. */
        href: Href;
        external?: false;
      }
    | {
        /** Endereço de fora das rotas do site (a API), usado como está. */
        href: string;
        /** Sem o `Link` do i18n: um `<a>` com o `href` cru. */
        external: true;
      }
  );

/**
 * Link com cara de botão (Carbon `Button` sobre o `Link` do i18n): para ação
 * que leva a outra página ("Criar comunidade", "Editar", "Entrar"). Com
 * `external`, o `href` é um endereço fora das rotas do site (o OAuth do
 * Discord, na API) e vai cru, sem prefixo de idioma.
 */
export function ButtonLink({
  children,
  kind = "primary",
  size = "md",
  renderIcon,
  className,
  ...destination
}: Props) {
  const shared = { kind, size, renderIcon, className };
  if (destination.external) {
    return (
      <CarbonButton href={destination.href} {...shared}>
        {children}
      </CarbonButton>
    );
  }
  return (
    <CarbonButton
      as={IntlLink}
      // O Carbon tipa href como string; quem o recebe é o Link do i18n.
      href={destination.href as string}
      {...shared}
    >
      {children}
    </CarbonButton>
  );
}
