"use client";

import { type ReactNode, useMemo, useState } from "react";

import type { components } from "@/shared/api/schema";
import { type LiveMatch, LiveMatchContext } from "../hooks/use-live-match";
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
 * pelo passo marcado na hora. A do servidor volta a valer quando é tão nova
 * quanto (pelo histórico do veto): uma recarga que saiu antes do clique não
 * desfaz o passo.
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
  const base = local && isNewer(local, initial) ? local : initial;
  const value = useMemo<LiveMatch>(
    () => ({
      detail: optimistic ? { ...base, veto: optimistic } : base,
      viewerId,
      bansPerSeries,
      vetoOrder,
      adopt: (detail) => {
        setLocal(detail);
        setOptimistic(null);
      },
      preview: setOptimistic,
    }),
    [base, optimistic, viewerId, bansPerSeries, vetoOrder],
  );
  return <LiveMatchContext value={value}>{children}</LiveMatchContext>;
}
