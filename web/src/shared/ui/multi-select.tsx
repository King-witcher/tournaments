"use client";

import type {
  FilterableMultiSelectProps,
  MultiSelectProps,
} from "@carbon/react";
import { useLocale, useTranslations } from "next-intl";
import { type ReactNode, useId } from "react";

import { useListBoxTranslation } from "@/shared/hooks/use-list-box-translation";
import {
  MultiSelect as CarbonMultiSelect,
  FilterableMultiSelect,
} from "./carbon/multi-select";

type Props<Item> = Omit<
  MultiSelectProps<Item>,
  | "id"
  | "titleText"
  | "label"
  | "onChange"
  | "translateWithId"
  | "locale"
  | "clearAnnouncement"
  | "clearSelectionDescription"
  | "clearSelectionText"
  // Os dois tipos do Carbon recebem coisas diferentes aqui.
  | "downshiftProps"
> &
  Pick<
    FilterableMultiSelectProps<Item>,
    "placeholder" | "filterItems" | "onInputValueChange" | "inputProps"
  > & {
    /** Rótulo visível em cima do campo (`hideLabel` o esconde da tela). */
    titleText: NonNullable<ReactNode>;
    /** Gerado quando não vem. */
    id?: string;
    /** O texto do campo antes da escolha; sem ele (e com `filterable`), o `placeholder`. */
    label?: NonNullable<ReactNode>;
    onChange?: (data: { selectedItems: Item[] }) => void;
    /** Com campo de texto para filtrar a lista longa (`FilterableMultiSelect`). */
    filterable?: boolean;
  };

/**
 * Escolha de vários itens num menu com caixas de seleção (`MultiSelect` do
 * Carbon); com `filterable`, ganha o campo de texto para filtrar a lista
 * longa (`FilterableMultiSelect`, com `placeholder`, `filterItems` e
 * `onInputValueChange`). Poucas opções, todas à vista: `CheckboxGroup`. O
 * total escolhido aparece na etiqueta do campo, que limpa tudo. Um item com
 * `isSelectAll: true` vira o "selecionar todos". Os itens são ordenados no
 * idioma da interface, com os escolhidos em cima ao reabrir
 * (`selectionFeedback`). Não entra no `FormData`: guarde os escolhidos no
 * estado. Gera o `id` e traduz os rótulos de abrir o menu e de limpar e os
 * avisos do leitor de tela.
 */
export function MultiSelect<Item>({
  id,
  filterable = false,
  label,
  onChange,
  placeholder,
  filterItems,
  onInputValueChange,
  inputProps,
  itemToElement,
  ...props
}: Props<Item>) {
  const t = useTranslations("Common.multiSelect");
  const locale = useLocale();
  const translate = useListBoxTranslation();
  const generated = useId();
  const owned = {
    id: id ?? generated,
    translateWithId: translate,
    locale,
    clearSelectionDescription: t("selected"),
    clearSelectionText: t("clearHint"),
  };
  if (filterable) {
    return (
      <FilterableMultiSelect<Item>
        {...owned}
        {...props}
        itemToElement={itemToElement ?? undefined}
        placeholder={placeholder}
        filterItems={filterItems}
        onInputValueChange={onInputValueChange}
        inputProps={inputProps}
        onChange={onChange}
      />
    );
  }
  return (
    <CarbonMultiSelect<Item>
      {...owned}
      {...props}
      itemToElement={itemToElement}
      label={label ?? placeholder ?? ""}
      clearAnnouncement={t("cleared")}
      onChange={
        onChange &&
        ((data) => onChange({ selectedItems: data.selectedItems ?? [] }))
      }
    />
  );
}
