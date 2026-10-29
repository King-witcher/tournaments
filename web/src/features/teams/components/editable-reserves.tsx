"use client";

import { useTranslations } from "next-intl";

import { shownNickname } from "@/features/members/lib/nickname";
import type { FormationBoardState } from "../hooks/use-formation-board";
import { PlayerLine } from "./player-line";
import { PlayerMenu } from "./player-menu";
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
 * cada reserva se arrasta para um time (ou para "Novo time") ou vai pelo ⋯.
 */
export function EditableReserves({
  tenant,
  tournament,
  board,
  checkIn,
}: Props) {
  const tMember = useTranslations("Member");
  const { formation } = board;
  const checkedIn = new Set(formation.checked_in);

  return (
    <ReservesBlock
      count={formation.reserves.length}
      {...board.dropProps("reserve")}
    >
      {formation.reserves.map((p) => {
        const name = shownNickname(p.nickname, tMember("deletedAccount"));
        return (
          <PlayerLine
            key={p.user_id}
            {...board.dragProps(p.user_id, "reserve")}
            handle
            tenant={tenant}
            userId={p.user_id}
            nickname={p.nickname}
            avatarUrl={p.avatar_url}
            checkedIn={checkIn ? checkedIn.has(p.user_id) : undefined}
            left={p.left}
            actions={
              <PlayerMenu
                tenant={tenant}
                tournament={tournament}
                formation={formation}
                userId={p.user_id}
                name={name}
                from="reserve"
                captain={false}
                onDone={board.settle}
              />
            }
          />
        );
      })}
    </ReservesBlock>
  );
}
