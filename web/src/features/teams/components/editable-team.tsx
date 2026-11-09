"use client";

import type { components } from "@/shared/api/schema";
import type { FormationBoardState } from "../hooks/use-formation-board";
import { useFormationDrop } from "../hooks/use-formation-drop";
import { DraggablePlayerLine } from "./draggable-player-line";
import { RenameTeamButton } from "./rename-team-button";
import { TeamBlock } from "./team-block";
import { TeamState } from "./team-state";

type Props = {
  tenant: string;
  tournament: string;
  team: components["schemas"]["Team"];
  board: FormationBoardState;
  /** O check-in aparece e conta (`checkInShown`). */
  checkIn: boolean;
};

/**
 * Um time na edição da formação: recebe jogadores arrastados (se tiver
 * vaga), cada jogador se arrasta e tem o ⋯, e o time tem "Renomear".
 */
export function EditableTeam({
  tenant,
  tournament,
  team,
  board,
  checkIn,
}: Props) {
  const { formation } = board;
  const drop = useFormationDrop(board, team.id);

  return (
    <TeamBlock
      team={team}
      state={
        <TeamState
          team={team}
          formation={formation}
          checkIn={checkIn}
          started={false}
        />
      }
      actions={
        <RenameTeamButton
          tenant={tenant}
          tournament={tournament}
          team={team}
          onDone={board.replace}
        />
      }
      emptySlots={formation.team_size - team.members.length}
      indentSlots
      {...drop}
    >
      {team.members.map((m) => (
        <DraggablePlayerLine
          key={m.user_id}
          tenant={tenant}
          tournament={tournament}
          board={board}
          player={m}
          from={team.id}
          captain={m.user_id === team.captain}
          checkIn={checkIn}
        />
      ))}
    </TeamBlock>
  );
}
