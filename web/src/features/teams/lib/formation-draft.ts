import type { components } from "@/shared/api/schema";
import {
  canMove,
  type Destination,
  type Formation,
  playerOf,
  type Spot,
  spotOf,
} from "./formation";

type Schemas = components["schemas"];
type Team = Schemas["Team"];
type Member = Schemas["TeamMember"];
/** A posição de cada inscrito na ordem de inscrição. */
type Registration = ReadonlyMap<number, number>;

/**
 * O que a organização mudou e ainda não salvou. O rascunho é a formação da API
 * com a lista de mudanças por cima (`applyEdit`, uma a uma): o tempo real só
 * troca a de baixo, e quem entrou cai na reserva, quem saiu some, sem apagar o
 * que foi arrastado.
 */
export type FormationEdit =
  | { kind: "move"; userId: number; to: Spot }
  /** Time novo, com um id local (negativo) que o `move` seguinte pode usar. */
  | { kind: "newTeam"; userId: number; teamId: number }
  | { kind: "captain"; userId: number };

/** Time que só existe no rascunho (`teamId` negativo). */
export function isDraftTeam(team: Team) {
  return team.id < 0;
}

function registrationOf(formation: Formation): Registration {
  return new Map(formation.registration_order.map((id, i) => [id, i]));
}

function byRegistration(registration: Registration) {
  const rank = (m: Member) => registration.get(m.user_id) ?? Infinity;
  return (a: Member, b: Member) => rank(a) - rank(b);
}

/** Os jogadores na ordem da API: o capitão primeiro, depois por inscrição. */
function ordered(team: Team, registration: Registration): Team {
  const members = [...team.members]
    .sort(byRegistration(registration))
    .sort(
      (a, b) =>
        Number(b.user_id === team.captain) - Number(a.user_id === team.captain),
    );
  return { ...team, members };
}

/**
 * Tira `userId` de onde ele está: o time que fica vazio some, e o capitão
 * que sai passa a capitania ao inscrito há mais tempo (a mesma regra da API).
 */
function without(
  formation: Formation,
  userId: number,
  registration: Registration,
): Formation {
  return {
    ...formation,
    reserves: formation.reserves.filter((p) => p.user_id !== userId),
    teams: formation.teams.flatMap((team) => {
      if (!team.members.some((m) => m.user_id === userId)) return [team];
      const members = team.members.filter((m) => m.user_id !== userId);
      if (members.length === 0) return [];
      const captain =
        team.captain === userId
          ? [...members].sort(byRegistration(registration))[0].user_id
          : team.captain;
      return [ordered({ ...team, members, captain }, registration)];
    }),
  };
}

/** Aplica uma mudança; devolve o mesmo objeto se ela não se aplica mais. */
export function applyEdit(
  formation: Formation,
  edit: FormationEdit,
): Formation {
  const player = playerOf(formation, edit.userId);
  const from = spotOf(formation, edit.userId);
  if (!player || from === undefined) return formation;
  const registration = registrationOf(formation);

  if (edit.kind === "captain") {
    const team = formation.teams.find((t) => t.id === from);
    if (!team || team.captain === edit.userId) return formation;
    return {
      ...formation,
      teams: formation.teams.map((t) =>
        t.id === from
          ? ordered({ ...t, captain: edit.userId }, registration)
          : t,
      ),
    };
  }

  const to: Destination = edit.kind === "newTeam" ? "new" : edit.to;
  if (!canMove(formation, from, to)) return formation;
  const rest = without(formation, edit.userId, registration);
  if (edit.kind === "newTeam") {
    const team: Team = {
      id: edit.teamId,
      name: null,
      seed: null,
      captain: edit.userId,
      withdrawn: false,
      members: [player],
    };
    return { ...rest, teams: [...rest.teams, team] };
  }
  if (edit.to === "reserve") {
    const reserves = [...rest.reserves, player].sort(
      byRegistration(registration),
    );
    return { ...rest, reserves };
  }
  return {
    ...rest,
    teams: rest.teams.map((team) =>
      team.id === edit.to
        ? ordered({ ...team, members: [...team.members, player] }, registration)
        : team,
    ),
  };
}

/** A formação de `base` com `edits`, uma por cima da outra. */
export function applyEdits(
  base: Formation,
  edits: readonly FormationEdit[],
): Formation {
  return edits.reduce<Formation>((f, edit) => applyEdit(f, edit), base);
}

/** A composição: quem está em que time, e quem é o capitão. */
function compositionKey(formation: Formation) {
  return formation.teams
    .map((team) => {
      const ids = team.members.map((m) => m.user_id).sort((a, b) => a - b);
      return `${team.id}:${team.captain}:${ids.join(",")}`;
    })
    .sort()
    .join("|");
}

/** As duas formações têm os mesmos times, jogadores e capitães. */
export function sameFormation(a: Formation, b: Formation) {
  return compositionKey(a) === compositionKey(b);
}

/** O corpo do `PUT .../teams/formation`: o rascunho inteiro. */
export function draftBody(formation: Formation): Schemas["FormationDraft"] {
  return {
    teams: formation.teams.map((team) => ({
      id: isDraftTeam(team) ? null : team.id,
      members: team.members.map((m) => m.user_id),
      captain: team.captain,
    })),
  };
}
