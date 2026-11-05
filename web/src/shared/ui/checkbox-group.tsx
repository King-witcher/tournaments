"use client";

import type { CheckboxGroupProps } from "@carbon/react";
import { type ReactNode, useId } from "react";

import {
  CheckboxGroup as CarbonCheckboxGroup,
  Checkbox,
} from "./carbon/checkbox";

type Option<V extends string> = {
  value: V;
  label: NonNullable<ReactNode>;
  /** Dica embaixo da opção. */
  helperText?: ReactNode;
  disabled?: boolean;
};

type Props<V extends string> = Omit<CheckboxGroupProps, "children"> & {
  options: Option<V>[];
  /** Os valores marcados. */
  value: V[];
  /** Entrega os marcados na ordem das opções. */
  onChange: (value: V[]) => void;
  /** Com ele, cada marcada entra no `FormData` (`name=valor`); base dos `id`. */
  name?: string;
};

/**
 * Várias escolhas entre poucas opções à vista (`CheckboxGroup` do Carbon,
 * com a `legend` em `legendText`); lista longa vai no `MultiSelect`, e uma
 * escolha só, no `RadioGroup`. `helperText`, `invalid` + `invalidText`,
 * `warn` + `warnText`, `disabled` e `readOnly` valem para o grupo todo;
 * `orientation="horizontal"` só para rótulos curtos. Monta as caixas a partir
 * de `options`, com `id` gerado, e `onChange` entrega a lista marcada.
 */
export function CheckboxGroup<V extends string>({
  options,
  value,
  onChange,
  name,
  ...props
}: Props<V>) {
  const generated = useId();
  const base = name ?? generated;
  return (
    <CarbonCheckboxGroup {...props}>
      {options.map((option) => (
        <Checkbox
          key={option.value}
          id={`${base}-${option.value}`}
          name={name}
          value={option.value}
          labelText={option.label}
          helperText={option.helperText}
          disabled={option.disabled}
          checked={value.includes(option.value)}
          onChange={(_event, { checked }) =>
            onChange(
              options
                .map((other) => other.value)
                .filter((other) =>
                  other === option.value ? checked : value.includes(other),
                ),
            )
          }
        />
      ))}
    </CarbonCheckboxGroup>
  );
}
