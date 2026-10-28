"use client";

import { ProgressIndicator, ProgressStep } from "@carbon/react";
import { useTranslations } from "next-intl";

import styles from "./progress-steps.module.scss";

type Step = {
  key: string;
  /** Nome do passo, já traduzido. */
  label: string;
  /** Linha curta embaixo do nome (uma data, "agora"). */
  secondaryLabel?: string;
};

type Props = {
  steps: Step[];
  /** Índice do passo atual; os anteriores aparecem como feitos. */
  current: number;
  /** O que a lista é, para leitor de tela ("Ciclo do torneio"). */
  label: string;
};

/**
 * Passos de um processo com o atual marcado (`ProgressIndicator` do Carbon),
 * só leitura: o ciclo do torneio. No celular fica vertical.
 */
export function ProgressSteps({ steps, current, label }: Props) {
  const t = useTranslations("Common.progress");
  const translate = (id: string) =>
    id.endsWith("complete") && !id.endsWith("incomplete")
      ? t("complete")
      : id.endsWith("current")
        ? t("current")
        : id.endsWith("invalid")
          ? t("invalid")
          : t("incomplete");
  const list = (vertical: boolean) => (
    <ProgressIndicator
      currentIndex={current}
      vertical={vertical}
      spaceEqually={!vertical}
      aria-label={label}
    >
      {steps.map((step) => (
        <ProgressStep
          key={step.key}
          label={step.label}
          secondaryLabel={step.secondaryLabel}
          translateWithId={translate}
        />
      ))}
    </ProgressIndicator>
  );
  // Um wrapper por orientação: o display:none fica nele, não na classe do
  // Carbon (que define o próprio display).
  return (
    <>
      <div className={styles.horizontal}>{list(false)}</div>
      <div className={styles.vertical}>{list(true)}</div>
    </>
  );
}
