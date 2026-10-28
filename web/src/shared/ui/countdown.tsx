"use client";

import { useEffect, useRef, useState } from "react";

import { useIsClient } from "@/shared/hooks/use-is-client";
import styles from "./countdown.module.scss";

type Props = {
  /** Instante final em ISO (UTC). */
  until: string;
  /** Chamado uma vez quando o tempo acaba. */
  onExpire?: () => void;
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
 * Tempo que falta até `until` (h:mm:ss ou m:ss), atualizado a cada segundo.
 * Até hidratar não mostra nada: o relógio do servidor não é o de quem vê.
 */
export function Countdown({ until, onExpire }: Props) {
  const isClient = useIsClient();
  const end = new Date(until).getTime();
  const [now, setNow] = useState(() => Date.now());
  const expired = useRef(false);

  useEffect(() => {
    const id = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(id);
  }, []);

  useEffect(() => {
    if (now >= end && !expired.current) {
      expired.current = true;
      onExpire?.();
    }
  }, [now, end, onExpire]);

  if (!isClient) return null;
  return (
    <time dateTime={until} className={styles.countdown}>
      {format(end - now)}
    </time>
  );
}
