import { Add, ArrowDown, ArrowUp, Close } from "@carbon/icons-react";
import { useTranslations } from "next-intl";
import { useEffect, useRef } from "react";

import { cn } from "@/shared/lib/cn";
import { ContainedListItem } from "@/shared/ui/contained-list";
import { IconButton } from "@/shared/ui/icon-button";
import type { Tiebreaker } from "../lib/tiebreakers";
import styles from "./tiebreaker-row.module.scss";

/** O controle que recebe o foco depois de mover, desligar ou ligar. */
export type FocusOn = "up" | "down" | "off" | "on";

/**
 * Para onde o foco vai quando o pedido não está na linha (seta na ponta,
 * desabilitada; linha que mudou de parte): a outra seta, e depois desligar.
 */
const FALLBACK: Record<FocusOn, FocusOn[]> = {
  up: ["up", "down", "off"],
  down: ["down", "up", "off"],
  off: ["off", "on"],
  on: ["on", "up", "off"],
};

type Props = {
  tiebreaker: Tiebreaker;
  /** A posição entre os ligados (1 é o primeiro); ausente no desligado. */
  position?: number;
  /** O torneio já começou: nada muda. */
  disabled: boolean;
  /** Ausente no primeiro ligado e nos desligados. */
  onUp?: () => void;
  /** Ausente no último ligado e nos desligados. */
  onDown?: () => void;
  /** Só no ligado. */
  onOff?: () => void;
  /** Só no desligado: vai para o fim dos ligados. */
  onOn?: () => void;
  /**
   * Muda a cada ação sobre este desempate: o foco vai para `focusOn` na linha
   * da posição nova (ela muda de lugar no DOM, e o foco se perderia).
   */
  focusToken?: number;
  focusOn?: FocusOn;
};

/**
 * Um desempate na lista do formulário: ligado, a posição, o nome e as ações
 * (subir, descer, desligar); desligado, o nome e "ligar". Cada botão diz o
 * nome do desempate.
 */
export function TiebreakerRow({
  tiebreaker,
  position,
  disabled,
  onUp,
  onDown,
  onOff,
  onOn,
  focusToken,
  focusOn,
}: Props) {
  const t = useTranslations("TournamentForm.tiebreakers");
  const name = t(`inline.${tiebreaker}`);
  const on = position !== undefined;
  const up = useRef<HTMLButtonElement>(null);
  const down = useRef<HTMLButtonElement>(null);
  const off = useRef<HTMLButtonElement>(null);
  const add = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (focusToken === undefined || focusOn === undefined) return;
    const refs = { up, down, off, on: add };
    const target = FALLBACK[focusOn]
      .map((k) => refs[k].current)
      .find((b) => b && !b.disabled);
    target?.focus();
  }, [focusToken, focusOn]);

  return (
    <ContainedListItem
      action={
        on ? (
          <>
            <IconButton
              ref={up}
              label={t("up", { name })}
              disabled={disabled || !onUp}
              onClick={onUp}
            >
              <ArrowUp />
            </IconButton>
            <IconButton
              ref={down}
              label={t("down", { name })}
              disabled={disabled || !onDown}
              onClick={onDown}
            >
              <ArrowDown />
            </IconButton>
            <IconButton
              ref={off}
              label={t("off", { name })}
              disabled={disabled}
              onClick={onOff}
            >
              <Close />
            </IconButton>
          </>
        ) : (
          <IconButton
            ref={add}
            label={t("on", { name })}
            disabled={disabled}
            onClick={onOn}
          >
            <Add />
          </IconButton>
        )
      }
    >
      <span className={cn(styles.content, on && styles.three)}>
        <span className={styles.position}>
          {on && t("position", { position })}
        </span>
        <span className={on ? undefined : styles.off}>
          {t(`names.${tiebreaker}`)}
        </span>
      </span>
    </ContainedListItem>
  );
}
