import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];
export type Formation = Schemas["TeamFormation"];
type Team = Schemas["Team"];

/** Onde um inscrito está na formação: o id do time dele ou a reserva. */
export type Spot = number | "reserve";

/** Para onde um jogador pode ir: um time, a reserva ou um time novo. */
export type Destination = Spot | "new";

/**
 * Como um time está para a chave: completo (o tamanho do torneio e, com o
 * check-in valendo, todos com check-in), faltando jogadores ou faltando o
 * check-in de alguém. É a regra que a API confere ao iniciar
 * (`tournament.teams_incomplete`).
 */
export type TeamState = "complete" | "missing" | "checkInPending";

export function teamState(
  team: Team,
  formation: Formation,
  checkIn: boolean,
): TeamState {
  if (team.members.length < formation.team_size) return "missing";
  if (checkIn) {
    const done = new Set(formation.checked_in);
    if (team.members.some((m) => !done.has(m.user_id))) {
      return "checkInPending";
    }
  }
  return "complete";
}

/** Quantos times completos o torneio precisa para começar. */
export const MIN_TEAMS = 2;

/** Quantos times estão completos, quantos não e quantos jogadores estão de reserva. */
export function formationCounts(formation: Formation, checkIn: boolean) {
  const complete = formation.teams.filter(
    (team) => teamState(team, formation, checkIn) === "complete",
  ).length;
  return {
    complete,
    incomplete: formation.teams.length - complete,
    reserves: formation.reserves.length,
  };
}

/**
 * O check-in aparece e conta na formação: o torneio tem check-in, ainda não
 * começou e ele está aberto (ou já houve, e o check-in foi desligado e ligado
 * de novo).
 * Antes de abrir, todos estariam "sem check-in", o que não diz nada; depois
 * de iniciar, os times da chave já passaram pela regra.
 */
export function checkInShown(
  status: Schemas["TournamentStatus"],
  formation: Formation,
) {
  if (!formation.has_check_in) return false;
  if (status === "check_in") return true;
  return (
    (status === "draft" || status === "registration") &&
    formation.checked_in.length > 0
  );
}

/** Quantos entram no sorteio: com check-in, quem fez; sem, todos os inscritos. */
export function drawPool(formation: Formation) {
  if (formation.has_check_in) return formation.checked_in.length;
  return formation.teams.reduce(
    (count, team) => count + team.members.length,
    formation.reserves.length,
  );
}

/** Onde o inscrito está; `undefined` se não está na formação. */
export function spotOf(formation: Formation, userId: number): Spot | undefined {
  const team = formation.teams.find((t) =>
    t.members.some((m) => m.user_id === userId),
  );
  if (team) return team.id;
  return formation.reserves.some((p) => p.user_id === userId)
    ? "reserve"
    : undefined;
}

/**
 * Se quem está em `from` pode ir para `to`: para outro time só se ele tiver
 * vaga; para a reserva só quem está num time; para um time novo, quem não
 * está sozinho no próprio time (sozinho, nada mudaria).
 */
export function canMove(formation: Formation, from: Spot, to: Destination) {
  if (to === from) return false;
  if (to === "reserve") return from !== "reserve";
  if (to === "new") {
    if (from === "reserve") return true;
    const own = formation.teams.find((t) => t.id === from);
    return (own?.members.length ?? 0) > 1;
  }
  const team = formation.teams.find((t) => t.id === to);
  return team !== undefined && team.members.length < formation.team_size;
}
