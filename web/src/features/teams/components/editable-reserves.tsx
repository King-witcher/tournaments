"use client";

import type { FormationBoardState } from "../hooks/use-formation-board";
import { useFormationDrop } from "../hooks/use-formation-drop";
import { DraggablePlayerLine } from "./draggable-player-line";
import { ReservesBlock } from "./reserves-block";

type Props = {
  tenant: string;
  tournament: string;
  board: FormationBoardState;
  /** O check-in aparece e conta (`checkInShown`). */
  checkIn: boolean;
};

/**
 * Os reservas na edição da formação: recebem quem é arrastado de um time, e
 * cada reserva se arrasta para um time (ou para o time novo) ou vai pelo ⋯.
 */
export function EditableReserves({
  tenant,
  tournament,
  board,
  checkIn,
}: Props) {
  const { formation } = board;
  const drop = useFormationDrop(board, "reserve");

  return (
    <ReservesBlock count={formation.reserves.length} {...drop}>
      {formation.reserves.map((p) => (
        <DraggablePlayerLine
          key={p.user_id}
          tenant={tenant}
          tournament={tournament}
          board={board}
          player={p}
          from="reserve"
          captain={false}
          checkIn={checkIn}
        />
      ))}
    </ReservesBlock>
  );
}
