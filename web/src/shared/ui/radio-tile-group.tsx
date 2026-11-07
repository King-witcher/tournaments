"use client";

import type { TileGroupProps } from "@carbon/react";
import { type ReactNode, useId } from "react";

import { cn } from "@/shared/lib/cn";
import { FormGroup } from "./carbon/form";
import { RadioTile, Tile, TileGroup } from "./carbon/tile";
import styles from "./radio-tile-group.module.scss";
import { RequiredLabel } from "./required-label";

type Option<V extends string> = {
  value: V;
  /** O nome da opção, sem nada clicável dentro (o tile todo escolhe). */
  content: NonNullable<ReactNode>;
  /** Uma frase do que a opção faz, embaixo do nome, em texto de apoio. */
  description?: ReactNode;
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
  /**
   * Só leitura (o padrão "Read-only states" do Carbon): só a escolhida, num
   * tile sem interação, como o pool de mapas mostra só os mapas dele.
   */
  readOnly?: boolean;
  /**
   * Quantos tiles lado a lado, no máximo (uma coluna sem isso). As colunas
   * seguem a largura do grupo: `5` dá cinco na aba larga, três numa coluna
   * de 8 e uma no celular, nunca um tile sozinho na última linha.
   */
  columns?: 2 | 3 | 4 | 5;
};

/** O nome e, embaixo, o que a opção faz. */
function TileText({
  content,
  description,
}: Pick<Option<string>, "content" | "description">) {
  return (
    <span className={styles.text}>
      <span className={styles.title}>{content}</span>
      {description && <span className={styles.description}>{description}</span>}
    </span>
  );
}

/**
 * Escolha única entre tiles (`TileGroup` + `RadioTile` do Carbon), para
 * opções que precisam de mais que um rótulo: o nome e uma frase do que cada
 * uma faz (`description`), para comparar antes de escolher, como os formatos
 * de torneio; opções de uma linha vão no `RadioGroup`. Por baixo são rádios
 * nativos: com `name`, a escolha entra no `FormData`, e `required` põe a
 * estrela na `legend`. Monta os tiles a partir de `options`, com o respiro dos
 * exemplos do Carbon: empilhados, ou em fileiras com `columns` (os tiles da
 * mesma fileira ficam da mesma altura, como o Carbon põe tiles na grade).
 */
export function RadioTileGroup<V extends string>({
  legend,
  options,
  value,
  onChange,
  name,
  required,
  readOnly = false,
  columns,
  ...props
}: Props<V>) {
  const generated = useId();
  const groupName = name ?? generated;
  const label = <RequiredLabel required={required}>{legend}</RequiredLabel>;

  if (readOnly) {
    const chosen = options.find((option) => option.value === value);
    return (
      <FormGroup legendText={label}>
        {chosen && (
          <Tile>
            <TileText
              content={chosen.content}
              description={chosen.description}
            />
          </Tile>
        )}
      </FormGroup>
    );
  }

  return (
    <div className={cn(styles.group, columns && styles[`columns${columns}`])}>
      <TileGroup<V>
        {...props}
        name={groupName}
        // O Carbon tipa a `legend` como texto, mas desenha qualquer nó: a
        // estrela do obrigatório precisa dele.
        legend={label as unknown as string}
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
            <TileText
              content={option.content}
              description={option.description}
            />
          </RadioTile>
        ))}
      </TileGroup>
    </div>
  );
}
