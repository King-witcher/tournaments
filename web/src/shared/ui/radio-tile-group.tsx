"use client";

import { RadioTile, TileGroup, type TileGroupProps } from "@carbon/react";
import { type ReactNode, useId } from "react";

import styles from "./radio-tile-group.module.scss";
import { RequiredLabel } from "./required-label";

type Option<V extends string> = {
  value: V;
  /** O conteúdo do tile, sem nada clicável dentro (o tile todo escolhe). */
  content: NonNullable<ReactNode>;
  disabled?: boolean;
};

type Props<V extends string> = Omit<
  TileGroupProps<V>,
  | "children"
  | "legend"
  | "name"
  | "onChange"
  | "valueSelected"
  | "defaultSelected"
> & {
  /** A pergunta do grupo (vira a `legend`). */
  legend: NonNullable<ReactNode>;
  options: Option<V>[];
  /** A escolhida; `undefined` antes da primeira escolha. */
  value: V | undefined;
  onChange: (value: V) => void;
  /** Para o `FormData`; gerado quando não vem. */
  name?: string;
};

/**
 * Escolha única entre tiles (`TileGroup` + `RadioTile` do Carbon), para
 * opções que precisam de mais que um rótulo (título e descrição, como os
 * formatos de torneio); opções de uma linha vão no `RadioGroup`. Por baixo
 * são rádios nativos: com `name`, a escolha entra no `FormData`, e
 * `required` põe a estrela na `legend`. Monta os tiles a partir de
 * `options`, empilhados com o respiro dos exemplos do Carbon.
 */
export function RadioTileGroup<V extends string>({
  legend,
  options,
  value,
  onChange,
  name,
  required,
  ...props
}: Props<V>) {
  const generated = useId();
  const groupName = name ?? generated;
  return (
    <div className={styles.group}>
      <TileGroup<V>
        {...props}
        name={groupName}
        // O Carbon tipa a `legend` como texto, mas desenha qualquer nó: a
        // estrela do obrigatório precisa dele.
        legend={
          (
            <RequiredLabel required={required}>{legend}</RequiredLabel>
          ) as unknown as string
        }
        required={required}
        valueSelected={value}
        onChange={(selection) => onChange(selection)}
      >
        {options.map((option) => (
          <RadioTile
            key={option.value}
            id={`${groupName}-${option.value}`}
            value={option.value}
            disabled={option.disabled}
          >
            {option.content}
          </RadioTile>
        ))}
      </TileGroup>
    </div>
  );
}
