"use client";

import {
  Pagination,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableHeader,
  TableRow,
  TableToolbar,
  TableToolbarContent,
  TableToolbarSearch,
} from "@carbon/react";
import { useTranslations } from "next-intl";
import { type MouseEvent, type ReactNode, useState } from "react";

import { type Href, useRouter } from "@/i18n/navigation";
import { cn } from "@/shared/lib/cn";
import styles from "./data-table.module.scss";
import { EmptyState } from "./empty-state";
import { Link } from "./link";

export type DataTableColumn = {
  key: string;
  header: ReactNode;
  /** Some no celular (até `md`). Deixe sem para a principal e a de estado. */
  secondary?: boolean;
};

export type DataTableRow = {
  id: string;
  /** O conteúdo de cada coluna, pela `key` (texto, `StatusTag`, `Avatar`…). */
  cells: Record<string, ReactNode>;
  /** Texto em que a busca procura (nome, apelido…); sem ele, a linha não é filtrada. */
  search?: string;
  /** A linha abre esta página: a célula principal vira link. */
  href?: Href;
  /** Ações da linha: um `OverflowActions` (ou um botão). */
  actions?: ReactNode;
};

type Props = {
  columns: DataTableColumn[];
  rows: DataTableRow[];
  /** O que a tabela é, para leitor de tela (e título, com `title`). */
  label: string;
  title?: ReactNode;
  description?: ReactNode;
  /** A coluna que vira link quando a linha tem `href`; padrão, a primeira. */
  primaryColumn?: string;
  /** Ação da barra (ex.: "Gerar link", primária), à direita da busca; a barra fica no lugar também sem linhas. */
  toolbar?: ReactNode;
  /** Sem nenhuma linha: o que falta e o próximo passo (a ação da barra, se houver, continua nela). */
  empty: { text: ReactNode; action?: ReactNode };
  size?: "sm" | "md" | "lg";
};

const PAGE_SIZES = [25, 50, 100];

/** Sem acento e em minúsculas: "Joao" acha "João". */
function normalize(text: string) {
  return text
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .toLowerCase();
}

/**
 * Lista grande (inscritos, membros, convites, torneios da comunidade):
 * `DataTable` do Carbon com busca na barra, paginação no cliente (25, 50 ou
 * 100 por página; a API não pagina), linha que abre página, ações por linha
 * e estado vazio com o próximo passo. As células chegam prontas (Server
 * Component pode montar as linhas). Carregando: `TableSkeleton`.
 */
export function DataTable({
  columns,
  rows,
  label,
  title,
  description,
  primaryColumn,
  toolbar,
  empty,
  size = "md",
}: Props) {
  const t = useTranslations("Common.table");
  const router = useRouter();
  const [query, setQuery] = useState("");
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(PAGE_SIZES[0]);
  const primary = primaryColumn ?? columns[0]?.key;
  const hasActions = rows.some((row) => row.actions !== undefined);
  // A barra fica montada quando as linhas vão de 0 para 1 e de volta (a ação
  // não perde o foco); a busca só aparece com linhas, e sem ação nem linhas
  // não há o que mostrar nela.
  const showBar = rows.length > 0 || toolbar !== undefined;

  const needle = normalize(query.trim());
  const filtered = needle
    ? rows.filter(
        (row) =>
          row.search === undefined || normalize(row.search).includes(needle),
      )
    : rows;
  const lastPage = Math.max(1, Math.ceil(filtered.length / pageSize));
  const current = Math.min(page, lastPage);
  const visible = filtered.slice((current - 1) * pageSize, current * pageSize);

  // Clique fora do link e dos botões da linha leva ao mesmo lugar do link.
  const openRow = (row: DataTableRow) => (event: MouseEvent) => {
    if (row.href === undefined) return;
    const target = event.target as HTMLElement;
    if (target.closest("a, button, input, select, textarea, [role='menu']"))
      return;
    router.push(row.href);
  };

  return (
    <TableContainer
      title={title}
      description={description}
      className={styles.container}
    >
      {showBar && (
        <TableToolbar aria-label={label}>
          <TableToolbarContent>
            {rows.length > 0 && (
              <TableToolbarSearch
                persistent
                labelText={t("search")}
                placeholder={t("search")}
                closeButtonLabelText={t("clearSearch")}
                onChange={(event) => {
                  setQuery(event === "" ? "" : event.target.value);
                  setPage(1);
                }}
              />
            )}
            {toolbar}
          </TableToolbarContent>
        </TableToolbar>
      )}
      {rows.length === 0 ? (
        <EmptyState action={empty.action}>{empty.text}</EmptyState>
      ) : (
        <Table aria-label={label} size={size} className={styles.table}>
          <TableHead>
            <TableRow>
              {columns.map((column) => (
                <TableHeader
                  key={column.key}
                  className={cn(column.secondary && styles.secondary)}
                >
                  {column.header}
                </TableHeader>
              ))}
              {hasActions && (
                <TableHeader className={styles.actions}>
                  <span className={styles.visuallyHidden}>{t("actions")}</span>
                </TableHeader>
              )}
            </TableRow>
          </TableHead>
          <TableBody>
            {visible.length === 0 ? (
              <TableRow>
                <TableCell colSpan={columns.length + (hasActions ? 1 : 0)}>
                  {t("noResults", { query: query.trim() })}
                </TableCell>
              </TableRow>
            ) : (
              visible.map((row) => (
                <TableRow
                  key={row.id}
                  className={cn(row.href !== undefined && styles.linkRow)}
                  onClick={openRow(row)}
                >
                  {columns.map((column) => (
                    <TableCell
                      key={column.key}
                      className={cn(column.secondary && styles.secondary)}
                    >
                      {row.href !== undefined && column.key === primary ? (
                        <Link href={row.href}>{row.cells[column.key]}</Link>
                      ) : (
                        row.cells[column.key]
                      )}
                    </TableCell>
                  ))}
                  {hasActions && (
                    <TableCell className={styles.actions}>
                      {row.actions}
                    </TableCell>
                  )}
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      )}
      {filtered.length > PAGE_SIZES[0] && (
        <Pagination
          page={current}
          pageSize={pageSize}
          pageSizes={PAGE_SIZES}
          totalItems={filtered.length}
          size={size}
          backwardText={t("previousPage")}
          forwardText={t("nextPage")}
          itemsPerPageText={t("itemsPerPage")}
          pageNumberText={t("pageNumber")}
          itemRangeText={(min, max, total) =>
            t("itemRange", { min, max, total })
          }
          pageRangeText={(_page, total) => t("pageRange", { total })}
          pageSelectLabelText={(total) => t("pageSelect", { total })}
          onChange={(next) => {
            setPage(next.page);
            setPageSize(next.pageSize);
          }}
        />
      )}
    </TableContainer>
  );
}
