import { Breadcrumb, BreadcrumbItem } from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { type Href, Link as IntlLink } from "@/i18n/navigation";
import styles from "./page-header.module.scss";

type Crumb = {
  href: Href;
  label: string;
};

type Props = {
  /** As páginas de cima, da raiz até a mãe (a atual é o título). */
  breadcrumbs?: Crumb[];
  title: ReactNode;
  /** `StatusTag`s ao lado do título. */
  tags?: ReactNode;
  /** Uma linha curta: formato · melhor de · início · inscritos N/máx. */
  meta?: ReactNode;
  /**
   * A ação primária e as secundárias (`Button`/`ButtonLink`/`ConfirmButton`),
   * na ordem em que aparecem; só uma `primary`.
   */
  actions?: ReactNode;
  /** O que é raro ou destrutivo: um `OverflowActions`. */
  overflow?: ReactNode;
  /** Abaixo do cabeçalho, dentro dele (raro: uma descrição curta). */
  children?: ReactNode;
};

/** O destino identifica a migalha (dois rótulos iguais não colidem); o índice desempata destinos repetidos. */
function crumbKey({ href }: Crumb, index: number) {
  return `${index}-${typeof href === "string" ? href : JSON.stringify(href)}`;
}

/**
 * O topo de toda página: onde estou (trilha e título), o que é isto (tags e
 * metadado) e o que posso fazer agora (uma ação primária, as secundárias e o
 * menu ⋯). No celular as ações descem para baixo do título.
 */
export function PageHeader({
  breadcrumbs,
  title,
  tags,
  meta,
  actions,
  overflow,
  children,
}: Props) {
  const t = useTranslations("Common");
  return (
    <header className={styles.header}>
      {breadcrumbs && breadcrumbs.length > 0 && (
        <Breadcrumb aria-label={t("breadcrumb")}>
          {breadcrumbs.map((crumb, index) => (
            <BreadcrumbItem key={crumbKey(crumb, index)}>
              <IntlLink href={crumb.href}>{crumb.label}</IntlLink>
            </BreadcrumbItem>
          ))}
        </Breadcrumb>
      )}
      <div className={styles.titleRow}>
        <div className={styles.heading}>
          <h1 className={styles.title}>{title}</h1>
          {tags}
        </div>
        {(actions || overflow) && (
          <div className={styles.actions}>
            {actions}
            {overflow}
          </div>
        )}
      </div>
      {meta && <p className={styles.meta}>{meta}</p>}
      {children}
    </header>
  );
}
