"use client";

import { RadioButton, RadioButtonGroup } from "@carbon/react";
import { type ReactNode, useId } from "react";

import styles from "./radio-group.module.scss";

type Option<V extends string> = {
  value: V;
  label: ReactNode;
  disabled?: boolean;
};

type Props<V extends string> = {
  /** A pergunta do grupo (vira a `legend`). */
  legendText: ReactNode;
  options: Option<V>[];
  value: V;
  onChange: (value: V) => void;
  /** `horizontal` só para opções curtas (duas ou três palavras). */
  orientation?: "horizontal" | "vertical";
  helperText?: ReactNode;
  invalid?: boolean;
  invalidText?: ReactNode;
  disabled?: boolean;
  name?: string;
};

/**
 * Escolha única entre poucas opções visíveis (até ~5; mais que isso,
 * `Select`). Cada opção tem alvo de toque de 48px abaixo de `md`, e as
 * horizontais que não cabem quebram de linha.
 */
export function RadioGroup<V extends string>({
  legendText,
  options,
  value,
  onChange,
  orientation = "vertical",
  name,
  ...props
}: Props<V>) {
  const generated = useId();
  const groupName = name ?? generated;
  return (
    <RadioButtonGroup
      className={styles.group}
      legendText={legendText}
      name={groupName}
      orientation={orientation}
      valueSelected={value}
      onChange={(selected) => onChange(selected as V)}
      {...props}
    >
      {options.map((option) => (
        <RadioButton
          key={option.value}
          id={`${groupName}-${option.value}`}
          value={option.value}
          labelText={option.label}
          disabled={option.disabled}
        />
      ))}
    </RadioButtonGroup>
  );
}
