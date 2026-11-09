import type { ReactNode } from "react";

/** O parâmetro do endereço da Classificação que abre o Modal dos empates. */
export const OPEN_TIES_PARAM = "ties";

/** O que os "Decidir" de fora põem no endereço da Classificação. */
export const OPEN_TIES_QUERY = `${OPEN_TIES_PARAM}=open`;

/** A tabela da Classificação: recebe o foco quando o botão que abriu o Modal some. */
export const STANDINGS_TABLE_ID = "standings-table";

/** Um número da classificação, para decidir: o rótulo e o valor já formatado. */
export type TieNumber = { key: string; label: string; value: ReactNode };

/** Um dos empatados. */
export type TieTeam = {
  id: number;
  name: string;
  avatarUrl?: string | null;
  numbers: TieNumber[];
};

/** Um grupo empatado, com os times na ordem da chave. */
export type Tie = {
  /** Identifica o grupo: os times, na ordem da chave. */
  key: string;
  /** A melhor posição do grupo. */
  position: number;
  teams: TieTeam[];
};

/** O que a organização escolheu para um empate. */
export type TieDraft = { order: number[]; reason: string };

/** A chave de um grupo pendente vindo da API. */
export function tieKey(teamIds: number[]) {
  return teamIds.join("-");
}
