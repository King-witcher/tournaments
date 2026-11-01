"use client";

import { ComboBox as CarbonComboBox, type ComboBoxProps } from "@carbon/react";
import { type ReactNode, useId } from "react";

import { useListBoxTranslation } from "@/shared/hooks/use-list-box-translation";
import { RequiredLabel } from "./required-label";

type Props<Item> = Omit<
  ComboBoxProps<Item>,
  "id" | "titleText" | "translateWithId"
> & {
  /** Rótulo visível em cima do campo. */
  titleText: NonNullable<ReactNode>;
  /** Gerado quando não vem. */
  id?: string;
};

/**
 * `ComboBox` do Carbon: escolha única numa lista longa, em que a pessoa
 * digita para filtrar (o campo de texto e o menu juntos). Com
 * `allowCustomValue`, aceita também um valor fora da lista; `typeahead`
 * completa o texto com o primeiro item que bate. Lista curta: `Select`, ou
 * `Dropdown` para filtrar a página. Os itens são texto ou objetos com
 * `label` (ou passe `itemToString`). Com `name`, o texto do campo entra no
 * `FormData`; `required` põe a estrela no rótulo. Gera o `id` e traduz os
 * rótulos de abrir o menu e de limpar.
 */
export function ComboBox<Item>({
  id,
  titleText,
  required,
  ...props
}: Props<Item>) {
  const generated = useId();
  const translate = useListBoxTranslation();
  return (
    <CarbonComboBox<Item>
      id={id ?? generated}
      titleText={<RequiredLabel required={required}>{titleText}</RequiredLabel>}
      required={required}
      translateWithId={translate}
      {...props}
    />
  );
}
