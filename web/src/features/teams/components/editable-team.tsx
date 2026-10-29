"use client";

import { useTranslations } from "next-intl";

import { shownNickname } from "@/features/members/lib/nickname";
import type { components } from "@/shared/api/schema";
import type { FormationBoardState } from "../hooks/use-formation-board";
import { PlayerLine } from "./player-line";
import { PlayerMenu } from "./player-menu";
import { RenameTeamButton } from "./rename-team-button";
import { TeamBlock } from "./team-block";
import { TeamStateTag } from "./team-state-tag";

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
 * vaga), cada jogador se arrasta e tem o ⋯ ("Mover para…", "Tornar
 * capitão"), e o time tem "Renomear".
 */
export function EditableTeam({
  tenant,
  tournament,
  team,
  board,
  checkIn,
}: Props) {
  const tMember = useTranslations("Member");
  const { formation } = board;
  const checkedIn = new Set(formation.checked_in);

  return (
    <TeamBlock
      team={team}
      tag={
        <TeamStateTag
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
      {...board.dropProps(team.id)}
    >
      {team.members.map((m) => {
        const name = shownNickname(m.nickname, tMember("deletedAccount"));
        return (
          <PlayerLine
            key={m.user_id}
            {...board.dragProps(m.user_id, team.id)}
            handle
            tenant={tenant}
            userId={m.user_id}
            nickname={m.nickname}
            avatarUrl={m.avatar_url}
            captain={m.user_id === team.captain}
            checkedIn={checkIn ? checkedIn.has(m.user_id) : undefined}
            left={m.left}
            actions={
              <PlayerMenu
                tenant={tenant}
                tournament={tournament}
                formation={formation}
                userId={m.user_id}
                name={name}
                from={team.id}
                captain={m.user_id === team.captain}
                onDone={board.settle}
              />
            }
          />
        );
      })}
    </TeamBlock>
  );
}
