"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { useFormationBoard } from "../hooks/use-formation-board";
import type { Formation } from "../lib/formation";
import { DrawSection } from "./draw-section";
import { EditableReserves } from "./editable-reserves";
import { EditableTeam } from "./editable-team";
import { FormationBoard } from "./formation-board";
import { FormationSummary } from "./formation-summary";
import { NewTeamZone } from "./new-team-zone";
import { TeamsColumn } from "./teams-column";

type Props = {
  tenant: string;
  tournament: string;
  /** A formação que a página buscou. */
  initial: Formation;
  /** O check-in aparece e conta (`checkInShown`). */
  checkIn: boolean;
  /** Os links do resumo ("Ordem da chave", "Voltar à organização"). */
  links?: ReactNode;
};

/**
 * A formação dos times para a organização, antes de iniciar: o resumo, os
 * sorteios e os times e reservas, com arrastar entre eles (no computador) e
 * o ⋯ de cada jogador (em qualquer tela). Cada ação mostra na hora a
 * formação que a API devolveu; quando a página recarrega (outra pessoa
 * mexeu), vale a dela.
 */
export function FormationEditor({
  tenant,
  tournament,
  initial,
  checkIn,
  links,
}: Props) {
  const t = useTranslations("Team");
  const board = useFormationBoard(tenant, tournament, initial);
  const { formation } = board;

  return (
    <>
      <FormationSummary
        formation={formation}
        checkIn={checkIn}
        started={false}
        links={links}
      />
      <DrawSection
        tenant={tenant}
        tournament={tournament}
        formation={formation}
        onDrawn={board.replace}
      />
      <FormationBoard
        sticky
        teams={
          <TeamsColumn
            count={formation.teams.length}
            description={t("howToEdit")}
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
          <EditableReserves
            tenant={tenant}
            tournament={tournament}
            board={board}
            checkIn={checkIn}
          />
        }
      />
    </>
  );
}
