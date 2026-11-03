"use client";

import { formatCountdown, useCountdown } from "@/shared/hooks/use-countdown";
import styles from "./countdown.module.scss";

type Props = {
  /** Instante final em ISO (UTC). */
  until: string;
  /** Chamado uma vez quando o tempo acaba (de novo para cada `until` novo). */
  onExpire?: () => void;
  /**
   * A hora da API (ISO) em que o servidor montou a página que trouxe
   * `until`, como o `now` do `MatchDetail`: corrige um relógio errado de
   * verdade (mais de 3 s). Fica para a aba toda (`shared/lib/server-clock`),
   * que prefere as respostas de pedidos feitos do navegador.
   */
  serverNow?: string;
};

/**
 * Tempo que falta até `until` (h:mm:ss ou m:ss), atualizado a cada segundo,
 * pela hora da API quando alguma resposta a deu (`serverNow`). Até hidratar
 * não mostra nada: o relógio do servidor não é o de quem vê.
 */
export function Countdown({ until, onExpire, serverNow }: Props) {
  const left = useCountdown(until, { onExpire, serverNow });
  if (left == null) return null;
  return (
    <time dateTime={until} className={styles.countdown}>
      {formatCountdown(left)}
    </time>
  );
}
