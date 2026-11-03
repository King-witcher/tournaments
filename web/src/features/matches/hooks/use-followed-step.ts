"use client";

import { useEffect, useState } from "react";

import { focusIfLost } from "@/shared/lib/focus";
import { VETO_ID } from "../lib/match-target";

/** O foco está num controle do passo à vista (alguém lendo ou digitando nele). */
const focusInside = () => {
  const panel = document.getElementById(VETO_ID);
  const active = document.activeElement;
  return panel != null && active !== panel && panel.contains(active);
};

type Options<K> = {
  /** O passo que abre agora. */
  home: K;
  /** Uma ação do veto no ar: espera a resposta. */
  pending: string | null;
  isMine: () => boolean;
  clearMine: () => void;
  /** O passo ainda pede algo de quem vê. */
  asks: (key: K) => boolean;
};

/**
 * Qual passo está à vista. Abre no que `home` diz; quando ele muda, a tela só
 * segue sozinha se a mudança veio de uma ação de quem vê (e então o foco vai
 * junto) ou se ele estava nesse passo, que não pede mais nada dele, sem o foco
 * num controle dele; com uma ação do veto no ar, espera a resposta. Nos outros
 * casos fica onde está: o indicador mostra o estado do passo novo.
 */
export function useFollowedStep<K>({
  home,
  pending,
  isMine,
  clearMine,
  asks,
}: Options<K>) {
  const [view, setView] = useState({ home, shown: home });
  const [focusTurn, setFocusTurn] = useState(0);
  if (home !== view.home && pending == null) {
    const mine = isMine();
    const follow =
      mine || (view.shown === view.home && !asks(view.home) && !focusInside());
    setView({ home, shown: follow ? home : view.shown });
    if (mine) setFocusTurn((n) => n + 1);
  }
  // A mudança da ação de quem vê: o botão usado sumiu, o foco vai para o
  // passo novo.
  useEffect(() => {
    if (focusTurn === 0) return;
    clearMine();
    focusIfLost(VETO_ID);
    // O foco da ação do veto já pode estar no passo, sem ter rolado: o passo
    // novo começa do topo, se o topo ficou para cima.
    const panel = document.getElementById(VETO_ID);
    if (!panel) return;
    const margin = Number.parseFloat(
      getComputedStyle(panel).scrollMarginBlockStart,
    );
    if (panel.getBoundingClientRect().top < margin) panel.scrollIntoView();
  }, [focusTurn, clearMine]);
  return {
    shown: view.shown,
    show: (key: K) => setView({ ...view, shown: key }),
  };
}
