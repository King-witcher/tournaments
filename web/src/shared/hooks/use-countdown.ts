import { useEffect, useRef, useState } from "react";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { noteServerTime, serverTime } from "@/shared/lib/server-clock";

type Options = {
  /** Chamado uma vez quando o tempo acaba (de novo para cada `until` novo). */
  onExpire?: () => void;
  /**
   * A hora da API (ISO) em que o servidor montou a página que trouxe `until`:
   * corrige um relógio errado de verdade (`shared/lib/server-clock`).
   */
  serverNow?: string;
};

/** O tempo em h:mm:ss ou m:ss; negativo vale zero. */
export function formatCountdown(ms: number) {
  const total = Math.max(0, Math.ceil(ms / 1000));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = String(total % 60).padStart(2, "0");
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, "0")}:${seconds}`
    : `${minutes}:${seconds}`;
}

/**
 * Quantos ms faltam até `until`, a cada segundo, pela hora da API quando
 * alguma resposta a deu. `null` sem prazo e até hidratar: o relógio do
 * servidor não é o de quem vê. Renderiza quem o usa a cada segundo: fica na
 * folha que mostra o tempo, nunca acima de uma lista.
 */
export function useCountdown(
  until: string | null,
  { onExpire, serverNow }: Options = {},
) {
  const isClient = useIsClient();
  const [now, setNow] = useState(serverTime);
  // O prazo que já avisou: um `until` novo (o passo seguinte) avisa de novo.
  const expired = useRef<string | null>(null);
  const end = until == null ? null : new Date(until).getTime();

  useEffect(() => {
    if (serverNow === undefined) return;
    noteServerTime(serverNow);
    setNow(serverTime());
  }, [serverNow]);

  const running = until != null;
  useEffect(() => {
    if (!running) return;
    const id = window.setInterval(() => setNow(serverTime()), 1000);
    return () => window.clearInterval(id);
  }, [running]);

  useEffect(() => {
    if (end != null && now >= end && expired.current !== until) {
      expired.current = until;
      onExpire?.();
    }
  }, [now, end, until, onExpire]);

  return isClient && end != null ? end - now : null;
}
