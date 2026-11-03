"use client";

import type { ReactNode } from "react";

import {
  type ProgressStepItem,
  ProgressSteps,
} from "@/shared/ui/progress-steps";
import { VETO_ID } from "../lib/match-target";
import styles from "./veto-steps-frame.module.scss";

type Props = {
  steps: ProgressStepItem[];
  current: number;
  /** O que a lista é, para leitor de tela. */
  label: string;
  onSelect: (index: number) => void;
  selected: number;
  /** Entre o indicador e o passo: o que vale para todos os passos. */
  extra?: ReactNode;
  /** O passo à vista. */
  children: ReactNode;
};

/**
 * O indicador dos passos e, embaixo, o passo à vista, na série de um torneio e
 * na sala de veto. O passo recebe o foco depois de uma ação cujo botão some.
 * Com um passo só, sem indicador.
 */
export function VetoStepsFrame({
  steps,
  current,
  label,
  onSelect,
  selected,
  extra,
  children,
}: Props) {
  return (
    <div className={styles.steps}>
      {steps.length > 1 && (
        <ProgressSteps
          steps={steps}
          current={current}
          label={label}
          onSelect={onSelect}
          selected={selected}
        />
      )}
      {extra}
      {/* tabIndex -1: recebe o foco depois de um passo cujo botão some. */}
      <div id={VETO_ID} tabIndex={-1} className={styles.panel}>
        {children}
      </div>
    </div>
  );
}
