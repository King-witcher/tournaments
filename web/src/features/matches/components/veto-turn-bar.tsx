"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { formatCountdown, useCountdown } from "@/shared/hooks/use-countdown";
import { cn } from "@/shared/lib/cn";
import { Callout } from "@/shared/ui/callout";
import { ProgressBar } from "@/shared/ui/progress-bar";
import styles from "./veto-turn-bar.module.scss";

/** A partir de quando o tempo está acabando: 10 s ou, nos passos curtos, o último terço. */
const endingFrom = (seconds: number) => Math.min(10, Math.ceil(seconds / 3));

type Props = {
  /** A frase da vez ("Sua vez: banir um mapa."). */
  sentence: ReactNode;
  /** O que acontece no fim do tempo. */
  note: string;
  /** É com quem vê: só ele recebe o aviso de fim. */
  acting: boolean;
  deadline: string | null;
  /** Os segundos do passo inteiro, para a barra. */
  seconds: number;
  serverNow: string;
  onExpire: () => void;
};

/**
 * A vez e o tempo, presos embaixo do cabeçalho ao rolar a lista. É a folha
 * que renderiza a cada segundo (a grade, irmã dela, não). O `Callout` diz a
 * vez; o tempo fica à direita, com a barra no pé do bloco. Acabando o tempo de
 * quem age, o `Callout` passa a `warning` e o leitor de tela ouve o aviso uma
 * vez (`role="alert"`, já montado antes de receber o texto). O tempo em si não
 * é anunciado (`role="timer"`; a barra fica escondida do leitor de tela, que
 * algumas vezes lê cada mudança de uma `progressbar`). A frase é anunciada
 * quando a vez muda, porque o `Callout` não tem `aria-live`.
 */
export function VetoTurnBar({
  sentence,
  note,
  acting,
  deadline,
  seconds,
  serverNow,
  onExpire,
}: Props) {
  const t = useTranslations("Match.veto");
  const left = useCountdown(deadline, { onExpire, serverNow });
  const secondsLeft = left == null ? null : Math.max(0, left / 1000);
  const limit = endingFrom(seconds);
  const ending = secondsLeft != null && secondsLeft <= limit;
  const over = secondsLeft === 0;
  const warn = acting && ending;

  return (
    <div className={styles.bar}>
      <Callout
        kind={warn ? "warning" : "info"}
        lowContrast={!warn}
        className={styles.callout}
      >
        <div className={styles.text}>
          <p role="status" className={styles.sentence}>
            {sentence}
          </p>
          {deadline && (
            <p className={styles.note}>{over ? t("drawing") : note}</p>
          )}
        </div>
      </Callout>
      {deadline && left != null && (
        <time
          role="timer"
          aria-label={t("timeLeft")}
          dateTime={deadline}
          className={cn(styles.clock, warn && styles.warn)}
        >
          {formatCountdown(left)}
        </time>
      )}
      {deadline && (
        <div aria-hidden className={styles.time}>
          <ProgressBar
            // Uma barra por prazo: a determinada do Carbon não volta atrás.
            key={deadline}
            size="small"
            hideLabel
            label={t("timeLeft")}
            max={seconds}
            value={secondsLeft == null ? 0 : seconds - secondsLeft}
          />
        </div>
      )}
      {acting && (
        <span role="alert" className={styles.srOnly}>
          {ending ? t("ending", { seconds: limit }) : ""}
        </span>
      )}
    </div>
  );
}
