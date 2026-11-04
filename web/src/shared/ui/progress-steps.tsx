"use client";

import { cn } from "@/shared/lib/cn";
import styles from "./progress-steps.module.scss";
import { ProgressStepsList } from "./progress-steps-list";
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
   * (nome na cor de link, o atual também com o hover de link e o teclado) e
   * o clique devolve o índice; os seguintes não respondem. Sem isto, nenhum
   * passo responde.
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
  const list = (vertical: boolean) => (
    <ProgressStepsList
      steps={steps}
      current={current}
      label={label}
      vertical={vertical}
      onSelect={onSelect}
      selected={selected}
    />
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
            onSelect={(i) => i <= current && onSelect?.(i)}
          />
        ) : (
          list(true)
        )}
      </div>
    </div>
  );
}
