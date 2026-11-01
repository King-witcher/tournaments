"use client";

import { useLiveMatch } from "../hooks/use-live-match";
import { vetoPhase } from "../lib/match-steps";
import type { MatchTarget } from "../lib/match-target";
import { VetoError } from "./veto-error";
import { VetoTurn } from "./veto-turn";
import { VetoVoteBoard } from "./veto-vote-board";

type Props = { target: MatchTarget; number: number };

/**
 * O veto no passo da partida da vez: a votação do mapa dela (de quem é a
 * vez, o tempo e os mapas oferecidos) e o erro da última ação. Só quando o
 * veto está nas partidas (a votação da partida 1 com draft fica nos
 * Campeões): um cronômetro só na página.
 */
export function GameVeto({ target, number }: Props) {
  const { detail } = useLiveMatch();
  const veto = detail.veto;
  if (!veto || vetoPhase(detail) !== "games") return null;
  const game = veto.games.find((g) => g.game === number);
  const voting =
    veto.map_mode === "vote" && !game?.map && veto.turn?.action === "vote";
  return (
    <>
      {voting && <VetoTurn />}
      {voting && <VetoVoteBoard target={target} game={number} />}
      <VetoError />
    </>
  );
}
