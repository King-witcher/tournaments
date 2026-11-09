"use client";

import { useDraggable } from "@dnd-kit/core";
import { useTranslations } from "next-intl";

import { shownNickname } from "@/features/members/lib/nickname";
import { DragHandle } from "@/shared/ui/drag-handle";
import type { FormationBoardState } from "../hooks/use-formation-board";
import type { Spot } from "../lib/formation";
import { PlayerLine } from "./player-line";
import { PlayerMenu } from "./player-menu";

type Props = {
  tenant: string;
  board: FormationBoardState;
  player: {
    user_id: number;
    nickname: string;
    avatar_url?: string | null;
    left: boolean;
  };
  /** Onde ele está agora. */
  from: Spot;
  /** É o capitão do time. */
  captain: boolean;
  /** O check-in aparece e conta (`checkInShown`). */
  checkIn: boolean;
};

/**
 * Um jogador na edição da formação: arrasta pela alça e tem o ⋯ ("Mover
 * para…", "Tornar capitão"), a alternativa ao arrastar.
 */
export function DraggablePlayerLine({
  tenant,
  board,
  player,
  from,
  captain,
  checkIn,
}: Props) {
  const t = useTranslations("Team.drag");
  const tMember = useTranslations("Member");
  const { formation } = board;
  const name = shownNickname(player.nickname, tMember("deletedAccount"));
  const { attributes, listeners, setNodeRef, setActivatorNodeRef, isDragging } =
    useDraggable({
      id: player.user_id,
      disabled: board.pending,
      attributes: { roleDescription: t("roleDescription") },
    });

  return (
    <PlayerLine
      ref={setNodeRef}
      placeholder={isDragging}
      handle={
        <DragHandle
          ref={setActivatorNodeRef}
          dragging={isDragging}
          aria-label={t("handle", { name })}
          {...attributes}
          {...listeners}
        />
      }
      tenant={tenant}
      userId={player.user_id}
      nickname={player.nickname}
      avatarUrl={player.avatar_url}
      captain={captain}
      checkedIn={
        checkIn ? formation.checked_in.includes(player.user_id) : undefined
      }
      left={player.left}
      actions={
        <PlayerMenu
          formation={formation}
          name={name}
          from={from}
          captain={captain}
          onMove={(to) => board.moveTo(player.user_id, to)}
          onMakeCaptain={() => board.makeCaptain(player.user_id)}
        />
      }
    />
  );
}
