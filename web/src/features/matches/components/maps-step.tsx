"use client";

import type { MatchTarget } from "../lib/match-target";
import { VetoMapBoard } from "./veto-map-board";
import { VetoTurn } from "./veto-turn";
import { VetoUndo } from "./veto-undo";

type Props = {
  target: MatchTarget;
  /** Passo feito, à vista para ver como terminou: só a grade. */
  done?: boolean;
};

/** O passo Mapas (modo veto): de quem é a vez e o tempo, desfazer e a grade do pool. */
export function MapsStep({ target, done = false }: Props) {
  return (
    <>
      {!done && <VetoTurn />}
      {!done && <VetoUndo target={target} />}
      <VetoMapBoard target={target} />
    </>
  );
}
