/**
 * Alvos de foco da formação dos times (com `tabIndex={-1}`). Depois de mover
 * alguém, o menu dele some (a linha vai para outro bloco): o foco vai para o
 * bloco de destino. Depois de sortear, para a lista dos times.
 */
export const TEAMS_ID = "teams";

export const RESERVES_ID = "reserves";

/** O bloco de um time. */
export function teamBlockId(teamId: number) {
  return `team-${teamId}`;
}
