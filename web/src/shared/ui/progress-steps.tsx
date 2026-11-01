"use client";

import { ProgressIndicator, ProgressStep } from "@carbon/react";
import { useTranslations } from "next-intl";

import { cn } from "@/shared/lib/cn";
import styles from "./progress-steps.module.scss";
import { ProgressStepsMenu } from "./progress-steps-menu";

export type ProgressStepItem = {
  key: string;
  /** Nome do passo, já traduzido. */
  label: string;
  /** Linha curta embaixo do nome (uma data, "agora", um placar). */
  secondaryLabel?: string;
  /** Passo com problema (o ícone de alerta do Carbon, em vermelho). */
  invalid?: boolean;
};

type Props = {
  steps: ProgressStepItem[];
  /** Índice do passo atual; os anteriores aparecem como feitos. */
  current: number;
  /** O que a lista é, para leitor de tela ("Ciclo do torneio"). */
  label: string;
  /**
   * Escolher o que ver: com isto, os passos feitos e o atual viram botões
   * (nome na cor de link) e o clique devolve o índice; os seguintes não
   * respondem.
   */
  onSelect?: (index: number) => void;
  /** O passo à vista (com `onSelect`): em negrito, como uma aba ativa. */
  selected?: number;
};

/**
 * Passos de um processo com o atual marcado (`ProgressIndicator` do Carbon):
 * só leitura (o ciclo do torneio) ou, com `onSelect`, navegável (os passos
 * da série). Lado a lado enquanto cada passo tem os 128px do Carbon; sem
 * espaço, vertical e, navegável com mais de 4 passos (a lista vertical
 * empurraria o conteúdo para longe no celular), um `Dropdown` com os passos,
 * o estado de cada um e os seguintes desabilitados.
 *
 * ```tsx
 * <ProgressSteps steps={steps} current={2} label={t("steps")}
 *   onSelect={setViewing} selected={viewing} />
 * ```
 */
export function ProgressSteps({
  steps,
  current,
  label,
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
  const list = (vertical: boolean) => (
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
  const compact = onSelect != null && steps.length > 4;
  // A largura decide (container query, sem pular na hidratação): os dois
  // existem no HTML e um some por CSS.
  return (
    <div className={cn(styles.steps, styles[`count${steps.length}`])}>
      <div className={styles.horizontal}>{list(false)}</div>
      <div className={styles.fallback}>
        {compact ? (
          <ProgressStepsMenu
            steps={steps}
            current={current}
            label={label}
            selected={selected ?? current}
            onSelect={choose}
          />
        ) : (
          list(true)
        )}
      </div>
    </div>
  );
}
