"use client";

import { type ReactNode, useRef, useState } from "react";

import type { components } from "@/shared/api/schema";
import { LiveMatchContext } from "../hooks/use-live-match";
import { useVetoRunner } from "../hooks/use-veto-runner";
import { isNewer } from "../lib/veto";

type Schemas = components["schemas"];

/** Quanto tempo uma ação de quem vê espera a mudança que ela causa. */
const MINE_MS = 10_000;

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
 * passam por aqui, uma por vez, e todas as ações de quem vê (as do veto e
 * as das partidas) marcam a mudança seguinte como dele (`markMine`), para os
 * passos seguirem para o novo só nesse caso.
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
  // A hora da última ação de quem vê (0: nenhuma por atender); as funções
  // ficam as mesmas entre as renderizações (entram em efeitos).
  const mineAt = useRef(0);
  const [mine] = useState(() => ({
    markMine: () => {
      mineAt.current = Date.now();
    },
    isMine: () => Date.now() - mineAt.current < MINE_MS,
    clearMine: () => {
      mineAt.current = 0;
    },
  }));
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
    run: (...args: Parameters<typeof run>) => {
      mine.markMine();
      run(...args);
    },
    ...mine,
  };
  return <LiveMatchContext value={value}>{children}</LiveMatchContext>;
}
