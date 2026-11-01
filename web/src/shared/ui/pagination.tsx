"use client";

import { Pagination as CarbonPagination } from "@carbon/react";
import { useTranslations } from "next-intl";
import type { ComponentProps } from "react";

type Props = ComponentProps<typeof CarbonPagination>;

/**
 * Paginação de uma lista de dados (`Pagination` do Carbon): "por página",
 * o intervalo de itens, a página atual num select e as setas. Vai embaixo de
 * tabela ou lista longa (o `DataTable` já a põe). Para navegar entre páginas
 * de conteúdo (sem "por página"), `PaginationNav`. Todos os textos do Carbon
 * saem traduzidos; um texto passado por prop substitui o padrão.
 * Carregando: `PaginationSkeleton` do `@carbon/react`.
 */
export function Pagination(props: Props) {
  const t = useTranslations("Common");
  return (
    <CarbonPagination
      backwardText={t("table.previousPage")}
      forwardText={t("table.nextPage")}
      itemsPerPageText={t("table.itemsPerPage")}
      pageNumberText={t("table.pageNumber")}
      itemRangeText={(min, max, total) =>
        t("table.itemRange", { min, max, total })
      }
      itemText={(min, max) => t("pagination.items", { min, max })}
      pageRangeText={(_page, total) => t("table.pageRange", { total })}
      pageSelectLabelText={(total) => t("table.pageSelect", { total })}
      pageText={(page) => t("pagination.page", { page })}
      {...props}
    />
  );
}
