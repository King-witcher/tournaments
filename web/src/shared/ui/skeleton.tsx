import {
  ButtonSkeleton,
  DataTableSkeleton,
  SkeletonText,
  Stack,
  TextAreaSkeleton,
  TextInputSkeleton,
  Tile,
} from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import styles from "./skeleton.module.scss";

// Peças de esqueleto com a forma do que vai chegar: monte o `loading.tsx` de
// cada rota com elas (o mesmo `Page`, as mesmas seções), para a página não
// pular quando o conteúdo chega. Uma peça por página leva o `label`, lido
// pelo leitor de tela; as outras são só visuais.

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

/**
 * Uma aba carregando, dentro da casca (cabeçalho e abas já prontos): o
 * "Carregando…" só para leitor de tela, com o `label` da aba, e os esqueletos
 * com a forma dela, empilhados como os blocos da aba. Use no `loading.tsx` e
 * no `<Suspense>` de toda aba, para quem usa leitor de tela saber que a aba
 * está chegando.
 */
export function TabLoading({
  label,
  children,
}: {
  /** O que está carregando, lido pelo leitor de tela. */
  label: string;
  /** Os esqueletos na forma da aba. */
  children: ReactNode;
}) {
  return (
    <Stack gap={7}>
      <LoadingLabel label={label} />
      {children}
    </Stack>
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

/** Uma seção de formulário carregando: a legenda e os campos, com o espaço do `FormSection`. */
export function FormSectionSkeleton({
  heading = true,
  children,
}: {
  /** Com a legenda da seção em cima. */
  heading?: boolean;
  /** Os campos (`TextInputSkeleton`, `SelectSkeleton`, `TextAreaSkeleton`…). */
  children: ReactNode;
}) {
  return (
    <Stack gap={6}>
      {heading && <SkeletonText heading width="40%" />}
      {children}
    </Stack>
  );
}

/**
 * Um formulário carregando, na forma dele: as seções com os campos e o botão
 * do Carbon em esqueleto, para a página não pular quando ele chega. Cada
 * seção repete a legenda (`heading`), a frase (`text`), `fields` campos e,
 * com `textarea`, uma área de texto. Formulário com seções de formas
 * diferentes (o de torneio) monta as suas com `FormSectionSkeleton`.
 */
export function FormSkeleton({
  sections = 1,
  fields = 2,
  textarea = false,
  heading = false,
  text = false,
  tile = false,
  label,
}: {
  /** Quantas seções. */
  sections?: number;
  /** Quantos campos (rótulo e caixa) por seção. */
  fields?: number;
  /** Com uma área de texto depois dos campos de cada seção. */
  textarea?: boolean;
  /** Com o título da seção em cima. */
  heading?: boolean;
  /** Com uma linha de texto (a descrição) antes dos campos. */
  text?: boolean;
  /** Dentro de um `Tile` (entrar, cadastro). */
  tile?: boolean;
  /**
   * O que está carregando, lido pelo leitor de tela; padrão "Carregando…".
   * `false` quando outra peça da mesma página já diz (um aviso por página).
   */
  label?: string | false;
}) {
  const section = (
    <>
      {heading && <SkeletonText heading width="40%" />}
      {text && <SkeletonText width="80%" />}
      {Array.from({ length: fields }, (_, i) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: campos fixos, sem identidade.
        <TextInputSkeleton key={i} />
      ))}
      {textarea && <TextAreaSkeleton />}
    </>
  );
  const body =
    sections === 1 ? (
      <Stack gap={6}>
        {section}
        <ButtonSkeleton />
      </Stack>
    ) : (
      <Stack gap={7}>
        {Array.from({ length: sections }, (_, i) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: seções fixas, sem identidade.
          <Stack key={i} gap={6}>
            {section}
          </Stack>
        ))}
        <ButtonSkeleton />
      </Stack>
    );
  return (
    <div>
      {label !== false && <LoadingLabel label={label} />}
      <div aria-hidden>{tile ? <Tile>{body}</Tile> : body}</div>
    </div>
  );
}
