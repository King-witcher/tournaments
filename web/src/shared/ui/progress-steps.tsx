"use client";

import { ProgressIndicator, ProgressStep } from "@carbon/react";
import { useTranslations } from "next-intl";

import { cn } from "@/shared/lib/cn";
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
  /**
   * Escolher um passo feito para ver como ele terminou: com isto, os passos
   * feitos viram botões (nome na cor de link) e o clique devolve o índice. O
   * atual e os seguintes não respondem; a volta ao atual fica com quem usa
   * (um botão "Voltar" no conteúdo).
   */
  onSelect?: (index: number) => void;
  /** O passo feito que está à vista (com `onSelect`): marcado como escolhido. */
  selected?: number;
};

/**
 * Passos de um processo com o atual marcado (`ProgressIndicator` do Carbon):
 * só leitura (o ciclo do torneio) ou, com `onSelect`, com os passos feitos
 * clicáveis (os passos da série). No celular fica vertical.
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
  // O Carbon põe o clique em todo passo que não é o atual: os seguintes saem
  // do Tab e do mouse, e o clique neles não chega.
  const choose = onSelect && ((i: number) => i < current && onSelect(i));
  const list = (vertical: boolean) => (
    <ProgressIndicator
      currentIndex={current}
      vertical={vertical}
      spaceEqually={!vertical}
      aria-label={label}
      onChange={choose}
    >
      {steps.map((step, i) => {
        const done = i < current;
        return (
          <ProgressStep
            key={step.key}
            label={step.label}
            secondaryLabel={step.secondaryLabel}
            translateWithId={translate}
            className={cn(
              onSelect && done && i !== selected && styles.selectable,
              onSelect && i === selected && styles.selected,
              onSelect && i > current && styles.inert,
            )}
            {...(onSelect && done ? { "aria-pressed": i === selected } : {})}
            {...(onSelect && i > current ? { tabIndex: -1 } : {})}
          />
        );
      })}
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
