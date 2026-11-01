"use client";

import { createContext, useContext } from "react";

import type { components } from "@/shared/api/schema";

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
  /** A série que uma ação devolveu (substitui o passo marcado na hora). */
  adopt: (detail: Schemas["MatchDetail"]) => void;
  /** Marca um passo na hora (`null` desfaz, no erro). */
  preview: (veto: Schemas["VetoView"] | null) => void;
};

export const LiveMatchContext = createContext<LiveMatch | null>(null);

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
