"use client";

import { ProgressIndicator, ProgressStep } from "@carbon/react";
import { useTranslations } from "next-intl";

import { cn } from "@/shared/lib/cn";
import type { ProgressStepItem } from "./progress-steps";
import styles from "./progress-steps-list.module.scss";

type Props = {
  steps: ProgressStepItem[];
  current: number;
  label: string;
  vertical: boolean;
  /** Navegável: os feitos e o atual abrem (o resto, como no `ProgressSteps`). */
  onSelect?: (index: number) => void;
  selected?: number;
};

/**
 * O `ProgressIndicator` do Carbon de um `ProgressSteps`, lado a lado ou
 * vertical, com os textos de estado traduzidos e, navegável, os passos
 * feitos e o atual como botões (nome na cor de link, o à vista em negrito)
 * e os seguintes inertes. Usado pelo `ProgressSteps`; não serve sozinho.
 */
export function ProgressStepsList({
  steps,
  current,
  label,
  vertical,
  onSelect,
  selected,
}: Props) {
  const t = useTranslations("Common.progress");
  const translate = (id: string) =>
    id.endsWith("complete") && !id.endsWith("incomplete")
      ? t("complete")
      : id.endsWith("current")
        ? t("current")
        : id.endsWith("invalid")
          ? t("invalid")
          : t("incomplete");
  const reachable = (i: number) => onSelect != null && i <= current;
  const choose = (i: number) => reachable(i) && onSelect?.(i);
  return (
    <ProgressIndicator
      currentIndex={current}
      vertical={vertical}
      spaceEqually={!vertical}
      aria-label={label}
      onChange={choose}
    >
      {steps.map((step, i) => (
        <ProgressStep
          key={step.key}
          label={step.label}
          secondaryLabel={step.secondaryLabel}
          invalid={step.invalid}
          translateWithId={translate}
          className={cn(
            reachable(i) && i !== selected && styles.selectable,
            reachable(i) && i === selected && styles.selected,
            onSelect && i > current && styles.inert,
          )}
          {...(reachable(i) ? { "aria-pressed": i === selected } : {})}
          {...(onSelect && i > current
            ? { tabIndex: -1, "aria-disabled": true }
            : {})}
          // O Carbon não põe o clique no passo atual: volta para ele.
          {...(onSelect && i === current
            ? { onClickCapture: () => choose(i) }
            : {})}
        />
      ))}
    </ProgressIndicator>
  );
}
