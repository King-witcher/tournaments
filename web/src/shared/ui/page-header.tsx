import { AspectRatio, Breadcrumb, BreadcrumbItem } from "@carbon/react";
import Image from "next/image";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { type Href, Link as IntlLink } from "@/i18n/navigation";
import { cn } from "@/shared/lib/cn";
import { BreadcrumbOverflow } from "./breadcrumb-overflow";
import styles from "./page-header.module.scss";

type Crumb = {
  href: Href;
  label: string;
};

type Props = {
  /** As páginas de cima, da raiz até a mãe (a atual é o título). */
  breadcrumbs?: Crumb[];
  title: ReactNode;
  /**
   * Antes do título, o logo do que a página é (`Avatar` `square` `lg`), como
   * o ícone do título no `PageHeader` do Carbon. Decorativo: o nome é o título.
   */
  logo?: ReactNode;
  /**
   * A capa (URL, 2:1) à direita do cabeçalho, como o `HeroImage` do
   * `PageHeader` do Carbon; só do `lg` para cima (no celular empurraria as
   * ações para baixo). Decorativa: o nome é o título.
   */
  image?: string | null;
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
 * menu ⋯). No celular as ações descem para baixo do título e o ⋯ vai para a
 * linha dele, no canto.
 */
export function PageHeader({
  breadcrumbs,
  title,
  logo,
  image,
  tags,
  meta,
  actions,
  overflow,
  children,
}: Props) {
  const t = useTranslations("Common");
  return (
    <header className={image ? styles.withImage : undefined}>
      <div className={styles.content}>
        <HeaderContent
          breadcrumbs={breadcrumbs}
          title={title}
          logo={logo}
          tags={tags}
          meta={meta}
          actions={actions}
          overflow={overflow}
          breadcrumbLabel={t("breadcrumb")}
        >
          {children}
        </HeaderContent>
      </div>
      {image && (
        <AspectRatio ratio="2x1" className={styles.image}>
          <Image
            src={image}
            alt=""
            fill
            unoptimized
            loading="eager"
            className={styles.cover}
          />
        </AspectRatio>
      )}
    </header>
  );
}

function HeaderContent({
  breadcrumbs,
  title,
  logo,
  tags,
  meta,
  actions,
  overflow,
  breadcrumbLabel,
  children,
}: Omit<Props, "image"> & { breadcrumbLabel: string }) {
  return (
    <>
      {breadcrumbs && breadcrumbs.length > 0 && (
        <Breadcrumb
          aria-label={breadcrumbLabel}
          className={styles.breadcrumb}
          noTrailingSlash
        >
          {/* No celular, como pede o Carbon: o ⋯ com as de cima e só a mãe. */}
          {breadcrumbs.length > 1 && (
            <BreadcrumbOverflow
              className={styles.phoneOnly}
              crumbs={breadcrumbs.slice(0, -1)}
            />
          )}
          {breadcrumbs.map((crumb, index) => (
            <BreadcrumbItem
              key={crumbKey(crumb, index)}
              className={cn(
                index < breadcrumbs.length - 1 && styles.desktopOnly,
              )}
            >
              <IntlLink href={crumb.href}>{crumb.label}</IntlLink>
            </BreadcrumbItem>
          ))}
        </Breadcrumb>
      )}
      <div className={styles.titleRow}>
        <div className={styles.heading}>
          {logo ? (
            // O logo fica ao lado do título também quando o título quebra.
            <div className={styles.named}>
              {logo}
              <h1 className={styles.title}>{title}</h1>
            </div>
          ) : (
            <h1 className={styles.title}>{title}</h1>
          )}
          {tags}
        </div>
        {(actions || overflow) && (
          <div className={styles.actions}>
            {actions && <div className={styles.buttons}>{actions}</div>}
            {overflow && <div className={styles.overflow}>{overflow}</div>}
          </div>
        )}
      </div>
      {meta && <p className={styles.meta}>{meta}</p>}
      {children}
    </>
  );
}
