import { useState } from "react";

import { focusIfLost } from "@/shared/lib/focus";
import { RESERVES_ID, teamBlockId } from "../lib/focus";
import {
  canMove,
  type Destination,
  type Formation,
  type Spot,
  spotOf,
} from "../lib/formation";
import { useMovePlayer } from "./use-move-player";

/** Quem está sendo arrastado e de onde saiu. */
export type DraggedPlayer = { userId: number; from: Spot };

/**
 * O estado da edição da formação: a formação de agora (a última que a API
 * devolveu, ou a da página quando ela recarrega) e quem está sendo arrastado.
 * `drop` move o jogador para onde pode ir; o resultado (andamento e erro)
 * aparece no bloco de destino (`useFormationDrop`).
 */
export function useFormationBoard(
  tenant: string,
  tournament: string,
  initial: Formation,
) {
  const [formation, setFormation] = useState(initial);
  // A página recarregou (outra pessoa mexeu, ou o tempo real): vale a dela.
  const [shown, setShown] = useState(initial);
  if (initial !== shown) {
    setShown(initial);
    setFormation(initial);
  }
  const [drag, setDrag] = useState<DraggedPlayer>();
  const [target, setTarget] = useState<Destination>();
  const moving = useMovePlayer(tenant, tournament);

  /** Grava a formação nova e leva o foco ao bloco onde o jogador ficou. */
  function settle(data: Formation, userId: number) {
    setFormation(data);
    const spot = spotOf(data, userId);
    focusIfLost(typeof spot === "number" ? teamBlockId(spot) : RESERVES_ID);
  }

  function drop(player: DraggedPlayer, to: Destination) {
    if (!canMove(formation, player.from, to)) return;
    setTarget(to);
    moving.move(player.userId, player.from, to, (data) =>
      settle(data, player.userId),
    );
  }

  return {
    formation,
    replace: setFormation,
    settle,
    drag,
    setDrag,
    drop,
    target,
    pending: moving.pending,
    error: moving.error,
  };
}

export type FormationBoardState = ReturnType<typeof useFormationBoard>;
