import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { components } from "@/shared/api/schema";
import type { Formation } from "../lib/formation";
import { FormationBoard } from "./formation-board";
import { PlayerLine } from "./player-line";
import { ReservesBlock } from "./reserves-block";
import { TeamBlock } from "./team-block";
import { TeamState } from "./team-state";
import { TeamsColumn } from "./teams-column";

type Team = components["schemas"]["Team"];

type Props = {
  tenant: string;
  formation: Formation;
  /** O check-in aparece e conta (`checkInShown`). */
  checkIn: boolean;
  /** Já começou: o seed antes do nome, sem o que falta nem vaga livre. */
  started: boolean;
  /** As ações de cada time (o ⋯ da organização, o "Renomear" do capitão). */
  teamActions?: (team: Team) => ReactNode;
};

/**
 * A formação só para ler: os times (nome, o que falta, jogadores com o capitão
 * marcado e as vagas livres) e os reservas, com o apelido levando à página da
 * pessoa.
 */
export function TeamList({
  tenant,
  formation,
  checkIn,
  started,
  teamActions,
}: Props) {
  const t = useTranslations("Team.teams");
  const checkedIn = new Set(formation.checked_in);
  const mark = (userId: number) =>
    checkIn ? checkedIn.has(userId) : undefined;

  return (
    <FormationBoard
      teams={
        <TeamsColumn count={formation.teams.length} empty={t("emptyReadOnly")}>
          {formation.teams.map((team) => (
            <TeamBlock
              key={team.id}
              team={team}
              showSeed={started}
              state={
                <TeamState
                  team={team}
                  formation={formation}
                  checkIn={checkIn}
                  started={started}
                />
              }
              actions={teamActions?.(team)}
              emptySlots={
                started ? 0 : formation.team_size - team.members.length
              }
            >
              {team.members.map((m) => (
                <PlayerLine
                  key={m.user_id}
                  tenant={tenant}
                  userId={m.user_id}
                  nickname={m.nickname}
                  avatarUrl={m.avatar_url}
                  link
                  captain={m.user_id === team.captain}
                  checkedIn={mark(m.user_id)}
                  left={m.left}
                />
              ))}
            </TeamBlock>
          ))}
        </TeamsColumn>
      }
      reserves={
        <ReservesBlock count={formation.reserves.length}>
          {formation.reserves.map((p) => (
            <PlayerLine
              key={p.user_id}
              tenant={tenant}
              userId={p.user_id}
              nickname={p.nickname}
              avatarUrl={p.avatar_url}
              link
              checkedIn={mark(p.user_id)}
              left={p.left}
            />
          ))}
        </ReservesBlock>
      }
    />
  );
}
