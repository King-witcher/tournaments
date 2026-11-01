"use client";

import { OverflowMenu, OverflowMenuItem } from "@carbon/react";
import { useLocale, useTranslations } from "next-intl";
import type { ComponentProps } from "react";

import { getPathname, type Href } from "@/i18n/navigation";

type Props = Partial<ComponentProps<typeof OverflowMenu>> & {
  /** As páginas de cima que não cabem, da raiz para baixo. */
  crumbs: { href: Href; label: string }[];
};

/**
 * O ⋯ que a trilha do `PageHeader` mostra no celular no lugar das páginas de
 * cima, como o Carbon pede para tela pequena (o ⋯ e só a página mãe). Cada
 * item é um link de verdade, com o prefixo do idioma. Vai direto dentro do
 * `BreadcrumbItem`, que o reconhece pelo nome e lhe passa o ícone e a
 * posição do menu da trilha (o resto das props vai para o `OverflowMenu`).
 */
export function BreadcrumbOverflow({ crumbs, ...props }: Props) {
  const t = useTranslations("Common");
  const locale = useLocale();
  return (
    <OverflowMenu
      aria-label={t("breadcrumbMore")}
      iconDescription={t("breadcrumbMore")}
      align="bottom"
      {...props}
    >
      {crumbs.map((crumb) => {
        const href = getPathname({ href: crumb.href, locale });
        return (
          <OverflowMenuItem key={href} itemText={crumb.label} href={href} />
        );
      })}
    </OverflowMenu>
  );
}

// O `BreadcrumbItem` do Carbon trata como menu da trilha o filho cujo nome
// contém "OverflowMenu".
BreadcrumbOverflow.displayName = "BreadcrumbOverflowMenu";
