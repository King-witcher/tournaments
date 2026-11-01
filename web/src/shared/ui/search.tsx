"use client";

import { Search as CarbonSearch, ExpandableSearch } from "@carbon/react";
import { useTranslations } from "next-intl";
import { type ComponentProps, useId } from "react";

type Props = Omit<
  ComponentProps<typeof CarbonSearch>,
  "id" | "closeButtonLabelText"
> & {
  /** Gerado quando não vem. */
  id?: string;
  /**
   * Só a lupa, que abre o campo ao clicar (`ExpandableSearch`): numa barra
   * de ferramentas sem espaço para o campo inteiro. A dica da lupa fechada
   * sai "Search", em inglês: o Carbon 1.119 a escreve fixa, sem prop.
   */
  expandable?: boolean;
};

/**
 * Busca do Carbon fora de tabela (a do `DataTable` já vem na barra dele):
 * filtra uma lista ou acha conteúdo na página. `labelText` é o nome do campo
 * para o leitor de tela (não aparece); `placeholder` vem "Buscar" quando não
 * passa outro. O "x" limpa o texto e chama `onClear`. Gera o `id` e traduz o
 * rótulo do "x".
 */
export function Search({
  id,
  expandable = false,
  placeholder,
  ...props
}: Props) {
  const t = useTranslations("Common.table");
  const generated = useId();
  const Component = expandable ? ExpandableSearch : CarbonSearch;
  return (
    <Component
      id={id ?? generated}
      placeholder={placeholder ?? t("search")}
      closeButtonLabelText={t("clearSearch")}
      {...props}
    />
  );
}
