"use client";

import { BreadcrumbItem, OverflowMenu, OverflowMenuItem } from "@carbon/react";
import { useLocale, useTranslations } from "next-intl";

import { getPathname, type Href } from "@/i18n/navigation";

type Props = {
  /** As páginas de cima que não cabem, da raiz para baixo. */
  crumbs: { href: Href; label: string }[];
  /** Classe do item da trilha (o `PageHeader` o mostra só no celular). */
  className?: string;
};

/**
 * O item da trilha com o ⋯ que o `PageHeader` mostra no celular no lugar das
 * páginas de cima, como o Carbon pede para tela pequena (o ⋯ e só a página
 * mãe). Cada item do menu é um link de verdade, com o prefixo do idioma. O
 * `BreadcrumbItem` vem junto, no mesmo componente de cliente: ele reconhece o
 * menu pelo nome do componente filho para lhe dar o ícone e a posição da
 * trilha, e um filho vindo do servidor chega sem esse nome no navegador (o
 * ⋯ virava o ⋮ comum, solto à esquerda, e a hidratação não batia).
 */
export function BreadcrumbOverflow({ crumbs, className }: Props) {
  const t = useTranslations("Common");
  const locale = useLocale();
  return (
    <BreadcrumbItem className={className}>
      <OverflowMenu
        aria-label={t("breadcrumbMore")}
        iconDescription={t("breadcrumbMore")}
        align="bottom"
      >
        {crumbs.map((crumb) => {
          const href = getPathname({ href: crumb.href, locale });
          return (
            <OverflowMenuItem key={href} itemText={crumb.label} href={href} />
          );
        })}
      </OverflowMenu>
    </BreadcrumbItem>
  );
}
