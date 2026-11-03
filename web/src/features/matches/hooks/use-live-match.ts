"use client";

import { createContext, useContext } from "react";

import type { components } from "@/shared/api/schema";
import type { VetoRequest } from "./use-veto-runner";

type Schemas = components["schemas"];

/** O que a área dos passos da série guarda (`LiveMatchProvider`). */
export type LiveMatch = {
  /** A série mais nova que a tela tem: do servidor, da resposta de uma ação ou o passo marcado na hora. */
  detail: Schemas["MatchDetail"];
  /** Quem vê (o cartão destaca o jogador dele no TDM). */
  viewerId: number | null;
  /** Bans de campeão de cada lado na série, do torneio. */
  bansPerSeries: number;
  /** A ordem do veto de mapas, do torneio. */
  vetoOrder: Schemas["VetoOrder"];
  /** A ação do veto no ar (`"ready"`, `"step"`, `"undo"`...), uma por vez. */
  pending: string | null;
  /** A mensagem da ação que falhou, até a vez mudar. */
  error: string | undefined;
  /** Manda uma ação do veto, com o passo marcado na hora (`optimistic`). */
  run: (
    key: string,
    request: VetoRequest<Schemas["MatchDetail"]>,
    optimistic?: Schemas["VetoView"],
  ) => void;
  /**
   * Marca a próxima mudança da série como de quem vê (antes do refresh de
   * uma ação dele): os passos seguem para o novo e levam o foco junto.
   */
  markMine: () => void;
  /** Uma ação de quem vê saiu há pouco (10 s) e ainda não foi atendida. */
  isMine: () => boolean;
  /** A mudança da ação de quem vê já foi atendida. */
  clearMine: () => void;
};

export const LiveMatchContext = createContext<LiveMatch | null>(null);

/**
 * `markMine` do `LiveMatchProvider`, para as ações das partidas (enviar,
 * confirmar, contestar, as da organização); fora dele, não faz nada.
 */
export function useMarkMine(): () => void {
  return useContext(LiveMatchContext)?.markMine ?? noop;
}

const noop = () => {};

/**
 * A série como a área dos passos a vê agora, para as folhas de cliente (os
 * tabuleiros do veto, os lados do cartão da partida). Só dentro do
 * `LiveMatchProvider`.
 */
export function useLiveMatch(): LiveMatch {
  const live = useContext(LiveMatchContext);
  if (!live) throw new Error("useLiveMatch fora do LiveMatchProvider");
  return live;
}
