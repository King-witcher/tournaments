"use client";

import { useEffect, useRef, useState } from "react";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { noteServerTime, serverTime } from "@/shared/lib/server-clock";
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

function format(ms: number) {
  const total = Math.max(0, Math.ceil(ms / 1000));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = String(total % 60).padStart(2, "0");
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, "0")}:${seconds}`
    : `${minutes}:${seconds}`;
}

/**
 * Tempo que falta até `until` (h:mm:ss ou m:ss), atualizado a cada segundo,
 * pela hora da API quando alguma resposta a deu (`serverNow`). Até hidratar
 * não mostra nada: o relógio do servidor não é o de quem vê.
 */
export function Countdown({ until, onExpire, serverNow }: Props) {
  const isClient = useIsClient();
  const end = new Date(until).getTime();
  const [now, setNow] = useState(serverTime);
  // O prazo que já avisou: um `until` novo (o passo seguinte) avisa de novo.
  const expired = useRef<string | null>(null);

  useEffect(() => {
    if (serverNow === undefined) return;
    noteServerTime(serverNow);
    setNow(serverTime());
  }, [serverNow]);

  useEffect(() => {
    const id = window.setInterval(() => setNow(serverTime()), 1000);
    return () => window.clearInterval(id);
  }, []);

  useEffect(() => {
    if (now >= end && expired.current !== until) {
      expired.current = until;
      onExpire?.();
    }
  }, [now, end, until, onExpire]);

  if (!isClient) return null;
  return (
    <time dateTime={until} className={styles.countdown}>
      {format(end - now)}
    </time>
  );
}
