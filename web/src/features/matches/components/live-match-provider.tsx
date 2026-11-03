"use client";

import { type ReactNode, useState } from "react";

import { useRealtimeHub } from "@/features/realtime/hooks/use-realtime-hub";
import { useRouter } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { type LiveMatch, LiveMatchContext } from "../hooks/use-live-match";
import { useMineMark } from "../hooks/use-mine-mark";
import { useSideNames } from "../hooks/use-side-names";

import { VetoBoardContext } from "../hooks/use-veto-board";
import { useVetoRunner } from "../hooks/use-veto-runner";
import type { MatchTarget } from "../lib/match-target";
import { matchVetoActions } from "../lib/match-veto-actions";
import { isNewer } from "../lib/veto";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
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
 * passos seguirem para o novo só nesse caso. Os tabuleiros leem o `VetoBoard`,
 * que serve também à sala de veto; o que é só da série fica no `LiveMatch`.
 */
export function LiveMatchProvider({
  target,
  initial,
  viewerId,
  bansPerSeries,
  vetoOrder,
  children,
}: Props) {
  const router = useRouter();
  const hub = useRealtimeHub();
  const [local, setLocal] = useState<Schemas["MatchDetail"] | null>(null);
  const [optimistic, setOptimistic] = useState<Schemas["VetoView"] | null>(
    null,
  );
  const shown = local && isNewer(local, initial) ? local : initial;
  const mine = useMineMark();
  const { pending, error, run } = useVetoRunner({
    initial,
    shown,
    adopt: (detail) => {
      setLocal(detail);
      setOptimistic(null);
    },
    preview: setOptimistic,
    headers: () => hub?.originHeaders() ?? {},
    refresh: () => router.refresh(),
  });
  const detail = optimistic ? { ...shown, veto: optimistic } : shown;
  const names = useSideNames(detail);
  const value: LiveMatch = {
    detail,
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
  return (
    <LiveMatchContext value={value}>
      <VetoBoardContext
        value={{
          veto: detail.veto ?? null,
          now: detail.now,
          mine: detail.viewer_side ?? null,
          bestOf: detail.summary.best_of,
          names,
          viewerId,
          canManage: detail.can_manage,
          hasUndo: true,
          hideEmptyMaps: false,
          pending,
          error,
          actions: matchVetoActions(target, value),
          refresh: () => router.refresh(),
        }}
      >
        {children}
      </VetoBoardContext>
    </LiveMatchContext>
  );
}
