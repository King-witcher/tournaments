"use client";

import { type ReactNode, useState } from "react";

import type { components } from "@/shared/api/schema";
import { LiveMatchContext } from "../hooks/use-live-match";
import { useVetoRunner } from "../hooks/use-veto-runner";
import { isNewer } from "../lib/veto";

type Schemas = components["schemas"];

type Props = {
  /** A série como o servidor a desenhou (muda a cada recarga). */
  initial: Schemas["MatchDetail"];
  viewerId: number | null;
  bansPerSeries: number;
  vetoOrder: Schemas["VetoOrder"];
  children: ReactNode;
};

/**
 * Guarda a série da área dos passos: começa com a do servidor; uma ação do
 * veto troca pela resposta dela (sem esperar a recarga da página) e, antes,
 * pelo passo marcado na hora. Vale a que a API montou por último (`now`):
 * uma recarga que saiu antes do clique não desfaz o passo. As ações do veto
 * passam por aqui, uma por vez.
 */
export function LiveMatchProvider({
  initial,
  viewerId,
  bansPerSeries,
  vetoOrder,
  children,
}: Props) {
  const [local, setLocal] = useState<Schemas["MatchDetail"] | null>(null);
  const [optimistic, setOptimistic] = useState<Schemas["VetoView"] | null>(
    null,
  );
  const shown = local && isNewer(local, initial) ? local : initial;
  const { pending, error, run } = useVetoRunner({
    initial,
    shown,
    adopt: (detail) => {
      setLocal(detail);
      setOptimistic(null);
    },
    preview: setOptimistic,
  });
  const value = {
    detail: optimistic ? { ...shown, veto: optimistic } : shown,
    viewerId,
    bansPerSeries,
    vetoOrder,
    pending,
    error,
    run,
  };
  return <LiveMatchContext value={value}>{children}</LiveMatchContext>;
}
