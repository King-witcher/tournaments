import { DataTableSkeleton } from "@carbon/react";
import { useTranslations } from "next-intl";

import styles from "./skeleton.module.scss";

// Peças de esqueleto com a forma do que vai chegar: monte o `loading.tsx` de
// cada rota com elas (o mesmo `Page`, as mesmas seções), para a página não
// pular quando o conteúdo chega. Uma peça por página leva o `label`, lido
// pelo leitor de tela; as outras são só visuais.

/**
 * @deprecated Use as peças com forma (`PageHeaderSkeleton`, `SectionSkeleton`,
 * `LoadingSection`, `TableSkeleton`, `TileGridSkeleton`) ou `SkeletonText`/
 * `SkeletonPlaceholder` do `@carbon/react`. A `className` antiga (Tailwind)
 * não faz mais nada.
 */
export function Skeleton(_: { className?: string }) {
  return <span aria-hidden className={styles.crumb} />;
}

/** O texto "Carregando…" só para leitor de tela. */
function LoadingLabel({ label }: { label?: string }) {
  const t = useTranslations("Common");
  return (
    <span role="status" className={styles.visuallyHidden}>
      {label ?? t("loading")}
    </span>
  );
}

/** O `PageHeader` carregando: trilha (com `breadcrumb`), título e metadado. */
export function PageHeaderSkeleton({
  breadcrumb = true,
  meta = false,
  label,
}: {
  breadcrumb?: boolean;
  meta?: boolean;
  /** O que está carregando, lido pelo leitor de tela; padrão "Carregando…". */
  label?: string;
}) {
  return (
    <div className={styles.header}>
      <LoadingLabel label={label} />
      {breadcrumb && <span aria-hidden className={styles.crumb} />}
      <span aria-hidden className={styles.title} />
      {meta && <span aria-hidden className={styles.meta} />}
    </div>
  );
}

/** As abas (`PageTabs`) carregando. */
export function TabsSkeleton() {
  return <span aria-hidden className={styles.tabs} />;
}

/** Uma `Section` carregando: o título e tiles (`tiles`) ou linhas (`rows`). */
export function SectionSkeleton({
  tiles,
  rows = 3,
}: {
  tiles?: number;
  rows?: number;
}) {
  return (
    <div aria-hidden className={styles.section}>
      <span className={styles.sectionTitle} />
      {tiles !== undefined ? (
        <TileGridSkeleton count={tiles} />
      ) : (
        Array.from({ length: rows }, (_, i) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: linhas fixas, sem identidade.
          <span key={i} className={styles.block} />
        ))
      )}
    </div>
  );
}

/**
 * Seção carregando, com o texto visível (lido também por leitor de tela) e
 * blocos com a altura de um item de lista (48px).
 */
export function LoadingSection({
  label,
  rows = 3,
}: {
  /** O que está carregando; sem ele, "Carregando…". */
  label?: string;
  rows?: number;
}) {
  const t = useTranslations("Common");
  return (
    <section role="status" className={styles.stack}>
      <p className={styles.label}>{label ?? t("loading")}</p>
      {Array.from({ length: rows }, (_, i) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: linhas fixas, sem identidade.
        <span key={i} aria-hidden className={styles.block} />
      ))}
    </section>
  );
}

/** Tabela carregando (`DataTableSkeleton`), com a barra de busca. */
export function TableSkeleton({
  columns = 3,
  rows = 5,
}: {
  columns?: number;
  rows?: number;
}) {
  return (
    <div aria-hidden>
      <DataTableSkeleton
        columnCount={columns}
        rowCount={rows}
        showHeader={false}
      />
    </div>
  );
}

/** Grade de tiles carregando (as listas "minhas"), na grade do `TileGrid`. */
export function TileGridSkeleton({ count = 3 }: { count?: number }) {
  return (
    <div aria-hidden className={styles.tileGrid}>
      {Array.from({ length: count }, (_, i) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: blocos fixos, sem identidade.
        <span key={i} className={styles.tile} />
      ))}
    </div>
  );
}

/**
 * Página carregando, genérica: cabeçalho, abas (com `tabs`) e uma seção.
 * Prefira montar o `loading.tsx` com as peças acima, na forma da página.
 */
export function PageSkeleton({
  label,
  tabs = false,
}: {
  label?: string;
  tabs?: boolean;
}) {
  return (
    <div className={styles.page}>
      <PageHeaderSkeleton label={label} />
      {tabs && <TabsSkeleton />}
      <SectionSkeleton rows={4} />
    </div>
  );
}
