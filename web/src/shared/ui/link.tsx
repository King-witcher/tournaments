"use client";

import type { CarbonIconType } from "@carbon/icons-react";
import { Link as CarbonLink } from "@carbon/react";
import type { ComponentProps, ReactNode } from "react";

import { type Href, Link as IntlLink } from "@/i18n/navigation";
import { cn } from "@/shared/lib/cn";
import styles from "./link.module.scss";

type Own = {
  href: Href;
  children: ReactNode;
  /** Dentro de um parágrafo: sublinhado sempre, sem a altura de toque. */
  inline?: boolean;
  size?: "sm" | "md" | "lg";
  /** Ícone à direita (ex.: `ArrowRight` para "ir para"). */
  renderIcon?: CarbonIconType;
  /** Troca de idioma mantendo a página (só o seletor de idioma usa). */
  locale?: ComponentProps<typeof IntlLink>["locale"];
  className?: string;
};

/** Além das próprias, as props de acessibilidade e de identificação de um `<a>` (`aria-label`, `aria-describedby`, `id`, `title`…). */
type Props = Own &
  Omit<
    ComponentProps<"a">,
    keyof Own | "onClick" | "target" | "rel" | "style" | "ref"
  >;

/**
 * Link do Carbon sobre o `Link` do i18n (mantém o prefixo de idioma). Solto,
 * fica azul e, no celular, com 48px de altura para o dedo; no meio do texto,
 * use `inline` (sublinhado). Repassa `aria-label` e as outras props de
 * acessibilidade ao link.
 */
export function Link({
  href,
  children,
  inline,
  size,
  renderIcon,
  locale,
  className,
  ...rest
}: Props) {
  return (
    <CarbonLink
      as={IntlLink}
      // O Carbon tipa href como string; quem o recebe é o Link do i18n.
      href={href as string}
      locale={locale}
      inline={inline}
      size={size}
      renderIcon={renderIcon}
      className={cn(!inline && styles.standalone, className)}
      {...rest}
    >
      {children}
    </CarbonLink>
  );
}
