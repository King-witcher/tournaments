"use client";

import { createContext, useContext } from "react";

import type { components } from "@/shared/api/schema";
import type { VetoChoice } from "../lib/veto-optimistic";

type Schemas = components["schemas"];

/** O que quem vê pode mandar: um passo, "Pronto" e, na série, o desfazer. */
export type VetoActions = {
  step: (choice: VetoChoice) => void;
  ready: () => void;
  start: () => void;
  undo: () => void;
  request: () => void;
  answer: (accept: boolean) => void;
};

/**
 * O que os tabuleiros do veto precisam, venha ele da série de um torneio
 * (`LiveMatchProvider`) ou de uma sala de veto (`VetoRoomProvider`).
 */
export type VetoBoard = {
  /** O veto mais novo que a tela tem, com o passo marcado na hora. */
  veto: Schemas["VetoView"] | null;
  /** A hora da API na resposta que a tela mostra, para o cronômetro. */
  now: string;
  /** O lado de quem vê, se joga. */
  mine: Schemas["Side"] | null;
  bestOf: number;
  /** O nome exibido de cada lado. */
  names: { a: string; b: string };
  /** Quem vê (o cartão destaca o jogador dele no TDM). */
  viewerId: number | null;
  /** Quem vê organiza: força o começo e desfaz direto. */
  canManage: boolean;
  /** Há desfazer: a sala de veto não tem. */
  hasUndo: boolean;
  /**
   * Sem mapa, o resumo do draft não guarda o lugar dele: na sala sem veto de
   * mapas, nunca vai haver um.
   */
  hideEmptyMaps: boolean;
  /** A ação do veto no ar (`"ready"`, `"step"`, `"undo"`...), uma por vez. */
  pending: string | null;
  /** A mensagem da ação que falhou, até a vez mudar. */
  error: string | undefined;
  actions: VetoActions;
  /** Busca de novo o que a tela mostra (o sorteio do fim do tempo). */
  refresh: () => void;
};

export const VetoBoardContext = createContext<VetoBoard | null>(null);

/** O veto como a tela o vê agora, para as folhas de cliente. */
export function useVetoBoard(): VetoBoard {
  const board = useContext(VetoBoardContext);
  if (!board) throw new Error("useVetoBoard fora de um VetoBoardContext");
  return board;
}
