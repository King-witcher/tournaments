"use client";

import type { CarbonIconType } from "@carbon/icons-react";
import { type ReactNode, useContext } from "react";

import { type Href, Link as IntlLink } from "@/i18n/navigation";
import type { ButtonKind, ButtonSize } from "./button";
import { Button as CarbonButton } from "./carbon/button";
import { DescribedByContext } from "./described-by-context";

type Props = {
  /** Destino, com o prefixo de idioma posto pelo `Link` do i18n. */
  href: Href;
  children: ReactNode;
  kind?: ButtonKind;
  size?: ButtonSize;
  renderIcon?: CarbonIconType;
  className?: string;
};

/**
 * Link com cara de botão (Carbon `Button` sobre o `Link` do i18n): para ação
 * que leva a outra página ("Criar comunidade", "Editar", "Entrar").
 */
export function ButtonLink({
  href,
  children,
  kind = "primary",
  size = "md",
  renderIcon,
  className,
}: Props) {
  const describedBy = useContext(DescribedByContext);
  return (
    <CarbonButton
      as={IntlLink}
      // O Carbon tipa href como string; quem o recebe é o Link do i18n.
      href={href as string}
      aria-describedby={describedBy}
      kind={kind}
      size={size}
      renderIcon={renderIcon}
      className={className}
    >
      {children}
    </CarbonButton>
  );
}
