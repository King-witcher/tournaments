"use client";

import type { CarbonIconType } from "@carbon/icons-react";
import type { ComponentProps, ReactNode } from "react";

import { getPathname, type Href, Link as IntlLink } from "@/i18n/navigation";
import { cn } from "@/shared/lib/cn";
import { Link as CarbonLink } from "./carbon/link";
import styles from "./link.module.scss";

type Own = {
  href: Href;
  children: ReactNode;
  /** Dentro de um parágrafo: sublinhado sempre, sem a altura de toque. */
  inline?: boolean;
  size?: "sm" | "md" | "lg";
  /** Ícone à direita (ex.: `ArrowRight` para "ir para"). */
  renderIcon?: CarbonIconType;
  /**
   * A mesma página em outro idioma (só o `LocaleSwitcher` usa): `<a>` comum,
   * que carrega tudo de novo. A navegação do Next troca o `<html>` sem rodar
   * o script do tema, e o tema escolhido se perde.
   */
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
      {...(locale
        ? { href: getPathname({ href, locale }), hrefLang: locale }
        : // O Carbon tipa href como string; quem o recebe é o Link do i18n.
          { as: IntlLink, href: href as string })}
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
