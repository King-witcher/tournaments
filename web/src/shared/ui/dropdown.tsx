"use client";

import type { DropdownProps } from "@carbon/react";
import { type ReactNode, useId } from "react";

import { useListBoxTranslation } from "@/shared/hooks/use-list-box-translation";
import { Dropdown as CarbonDropdown } from "./carbon/dropdown";

type Props<Item> = Omit<
  DropdownProps<Item>,
  "id" | "titleText" | "translateWithId"
> & {
  /** Rótulo visível em cima do campo (`hideLabel` o esconde da tela). */
  titleText: NonNullable<ReactNode>;
  /** Gerado quando não vem. */
  id?: string;
};

/**
 * `Dropdown` do Carbon: escolha única num menu desenhado pelo Carbon, para
 * filtrar ou ordenar o conteúdo da página (`type="inline"` numa barra, ao
 * lado do que ele filtra). Em formulário e no celular, prefira o `Select`,
 * que é nativo e entra no `FormData` (o `Dropdown` não manda valor); lista
 * longa, em que a pessoa digita para achar, é o `ComboBox`; duas opções, o
 * `RadioGroup`. `label` é o texto do campo antes da escolha; os itens são
 * texto ou objetos com `label` (ou passe `itemToString`), e item com
 * `disabled` fica desabilitado. Gera o `id` e traduz os rótulos de abrir e
 * fechar o menu.
 */
export function Dropdown<Item>({ id, ...props }: Props<Item>) {
  const generated = useId();
  const translate = useListBoxTranslation();
  return (
    <CarbonDropdown<Item>
      id={id ?? generated}
      translateWithId={translate}
      {...props}
    />
  );
}
