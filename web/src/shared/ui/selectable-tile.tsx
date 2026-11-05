"use client";

import { type ComponentProps, type ReactNode, useId } from "react";

import { SelectableTile as CarbonSelectableTile } from "./carbon/tile";

type Props = Omit<
  ComponentProps<typeof CarbonSelectableTile>,
  "id" | "selected" | "onChange" | "children" | "name" | "value"
> & {
  /** O conteúdo do tile, sem nada clicável dentro (o tile todo marca). */
  children: NonNullable<ReactNode>;
  selected: boolean;
  onChange: (selected: boolean) => void;
  /** Gerado quando não vem. */
  id?: string;
};

/**
 * `SelectableTile` do Carbon: um tile que marca e desmarca como uma caixa de
 * seleção, para opção que precisa de mais que um rótulo (título, descrição,
 * imagem). Vários lado a lado são escolhas independentes; para uma escolha
 * só entre tiles, `RadioTileGroup`. Não entra no `FormData`: guarde o
 * `selected` no estado. Controlado, com `onChange` que entrega o booleano, e
 * `id` gerado.
 */
export function SelectableTile({
  id,
  selected,
  onChange,
  disabled,
  ...props
}: Props) {
  const generated = useId();
  return (
    <CarbonSelectableTile
      id={id ?? generated}
      selected={selected}
      onChange={(_event, next) => onChange(next ?? !selected)}
      disabled={disabled}
      // O Carbon só tira o tile do Tab; o leitor de tela precisa ouvir que
      // ele está desabilitado.
      aria-disabled={disabled || undefined}
      {...props}
    />
  );
}
