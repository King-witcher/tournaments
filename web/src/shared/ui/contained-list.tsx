"use client";

import { useTranslations } from "next-intl";
import { type ComponentProps, useId } from "react";

import {
  ContainedList as CarbonContainedList,
  ContainedListItem as CarbonContainedListItem,
} from "./carbon/contained-list";
import { ExpandableSearch, Search } from "./carbon/search";

type Props = ComponentProps<typeof CarbonContainedList> & {
  /**
   * Busca que filtra os itens: a lupa que se expande no lugar do `action`
   * (o cabeçalho só existe com `label`) ou, com `persistent`, o campo sempre
   * aberto acima dos itens. Quem usa filtra pelo texto que chega em `onChange`.
   */
  search?: {
    onChange: (query: string) => void;
    persistent?: boolean;
    /** Nome do campo; padrão "Buscar". */
    label?: string;
  };
};

/**
 * Lista com título num espaço contido (`ContainedList` do Carbon): um card,
 * um painel lateral, um Modal. `kind` `on-page` (padrão, na página) ou
 * `disclosed` (dentro de algo que se abre, como um popover). `action` põe um
 * botão no cabeçalho; `search`, a busca com os textos traduzidos; `isInset`
 * recua as linhas divisórias. Lista em colunas na página inteira é a
 * `StructuredList`; grande, com paginação, o `DataTable`.
 */
export function ContainedList({
  search,
  action,
  size,
  children,
  ...rest
}: Props) {
  const t = useTranslations("Common.table");
  const id = useId();

  const field = search && {
    id,
    labelText: search.label ?? t("search"),
    placeholder: search.label ?? t("search"),
    closeButtonLabelText: t("clearSearch"),
    // A busca vai até lg; a lista tem também xl.
    size: size === undefined || size === "xl" ? ("lg" as const) : size,
    onChange: (event: { target: { value: string } }) =>
      search.onChange(event.target.value),
  };

  return (
    <CarbonContainedList
      size={size}
      action={
        field && !search.persistent ? <ExpandableSearch {...field} /> : action
      }
      {...rest}
    >
      {field && search.persistent && <Search {...field} />}
      {children}
    </CarbonContainedList>
  );
}

/**
 * Um item da `ContainedList`. O conteúdo é só texto ou componente sem ação;
 * a ação vai no `action` (`IconButton`, `OverflowActions`). Com `onClick`, o
 * item todo vira botão; para abrir uma página, use `LinkTile` ou `DataTable`
 * com `href`. `renderIcon` põe um ícone à esquerda.
 */
export function ContainedListItem(
  props: ComponentProps<typeof CarbonContainedListItem>,
) {
  return <CarbonContainedListItem {...props} />;
}
