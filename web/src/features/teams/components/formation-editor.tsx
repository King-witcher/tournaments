"use client";

import { useTranslations } from "next-intl";

import { Stack } from "@/shared/ui/carbon/stack";
import { Notice } from "@/shared/ui/notice";
import { useFormationBoard } from "../hooks/use-formation-board";
import { type Formation, formationCounts, MIN_TEAMS } from "../lib/formation";
import { DrawTeamsButton } from "./draw-teams-button";
import { EditableReserves } from "./editable-reserves";
import { EditableTeam } from "./editable-team";
import { FormationActions } from "./formation-actions";
import { FormationBoard } from "./formation-board";
import { FormationDnd } from "./formation-dnd";
import { NewTeamZone } from "./new-team-zone";
import { TeamsColumn } from "./teams-column";

type Props = {
  tenant: string;
  tournament: string;
  /** A formação que a página buscou. */
  initial: Formation;
  /** O check-in aparece e conta (`checkInShown`). */
  checkIn: boolean;
};

/**
 * A formação dos times para a organização, antes de iniciar: o aviso enquanto
 * faltam times completos e os times e reservas, com "Sortear times", o
 * arrastar entre eles e o ⋯ de cada jogador. Tudo muda um rascunho na tela,
 * que "Salvar times" grava de uma vez (`FormationActions`); quando a página
 * recarrega (outra pessoa mexeu), o rascunho é reaplicado sobre a formação
 * dela.
 */
export function FormationEditor({
  tenant,
  tournament,
  initial,
  checkIn,
}: Props) {
  const t = useTranslations("Team");
  const board = useFormationBoard(tenant, tournament, initial);
  const { formation } = board;
  const { complete } = formationCounts(formation, checkIn);

  return (
    <FormationDnd board={board}>
      <Stack gap={7}>
        {complete < MIN_TEAMS && (
          <Notice>{t("needTeams", { min: MIN_TEAMS })}</Notice>
        )}
        <FormationBoard
          sticky
          teams={
            <TeamsColumn
              count={formation.teams.length}
              action={
                <DrawTeamsButton
                  tenant={tenant}
                  tournament={tournament}
                  formation={formation}
                  unsaved={board.dirty}
                  onDrawn={board.replace}
                />
              }
              empty={t("teams.empty")}
              extra={<NewTeamZone board={board} />}
            >
              {formation.teams.map((team) => (
                <EditableTeam
                  key={team.id}
                  tenant={tenant}
                  tournament={tournament}
                  team={team}
                  board={board}
                  checkIn={checkIn}
                />
              ))}
            </TeamsColumn>
          }
          reserves={
            <EditableReserves tenant={tenant} board={board} checkIn={checkIn} />
          }
        />
        <FormationActions
          tenant={tenant}
          tournament={tournament}
          board={board}
        />
      </Stack>
    </FormationDnd>
  );
}
