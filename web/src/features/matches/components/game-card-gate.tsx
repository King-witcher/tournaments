"use client";

import type { ReactNode } from "react";

import { useLiveMatch } from "../hooks/use-live-match";

type Props = { number: number; children: ReactNode };

/**
 * Esconde o cartão da partida da vez de quem joga enquanto ele não tem nada
 * a mostrar (a votação do mapa no passo, sem mapa nem envio), pela série
 * mais nova da tela: o último voto, com a resposta do clique, já põe o mapa
 * e mostra o cartão, sem esperar a recarga da página.
 */
export function GameCardGate({ number, children }: Props) {
  const { detail } = useLiveMatch();
  const veto = detail.veto;
  const plays = detail.viewer_side != null;
  const reportable = !veto || veto.reportable;
  const map = veto?.games.find((g) => g.game === number)?.map;
  if (plays && !reportable && !map) return null;
  return children;
}
