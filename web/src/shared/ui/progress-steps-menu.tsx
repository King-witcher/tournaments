"use client";

import {
  CheckmarkOutline,
  CircleDash,
  Incomplete,
  Warning,
} from "@carbon/icons-react";

import { cn } from "@/shared/lib/cn";
import { Dropdown } from "./dropdown";
import type { ProgressStepItem } from "./progress-steps";
import styles from "./progress-steps-menu.module.scss";

type Props = {
  steps: ProgressStepItem[];
  /** Índice do passo atual; os seguintes ficam desabilitados. */
  current: number;
  /** O que a lista é ("Passos da série"): o rótulo do campo. */
  label: string;
  /** O passo à vista. */
  selected: number;
  onSelect: (index: number) => void;
};

type Item = ProgressStepItem & { index: number; disabled: boolean };

/**
 * Os passos num `Dropdown`, para quando o `ProgressSteps` navegável não cabe
 * lado a lado e a lista vertical empurraria o conteúdo (celular, muitos
 * passos): cada item com o ícone de estado do indicador (feito, atual,
 * alerta, por fazer), o nome e a linha curta; os seguintes, desabilitados.
 * Usado pelo `ProgressSteps`; não serve sozinho.
 */
export function ProgressStepsMenu({
  steps,
  current,
  label,
  selected,
  onSelect,
}: Props) {
  const items: Item[] = steps.map((step, index) => ({
    ...step,
    index,
    disabled: index > current,
  }));
  const render = (item: Item) => {
    const Icon = item.invalid
      ? Warning
      : item.index < current
        ? CheckmarkOutline
        : item.index === current
          ? Incomplete
          : CircleDash;
    return (
      <span className={styles.item}>
        <Icon
          aria-hidden
          className={cn(
            styles.icon,
            item.invalid && styles.invalid,
            item.index > current && styles.later,
          )}
        />
        <span className={styles.label}>{item.label}</span>
        {item.secondaryLabel && (
          <span className={styles.secondary}>{item.secondaryLabel}</span>
        )}
      </span>
    );
  };
  return (
    <Dropdown<Item>
      titleText={label}
      label={label}
      items={items}
      selectedItem={items[selected]}
      itemToString={(item) =>
        item
          ? [item.label, item.secondaryLabel].filter(Boolean).join(" · ")
          : ""
      }
      itemToElement={render}
      renderSelectedItem={render}
      onChange={({ selectedItem }) => {
        if (selectedItem) onSelect(selectedItem.index);
      }}
    />
  );
}
