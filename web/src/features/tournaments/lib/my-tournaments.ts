import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];
type Entry = Schemas["MyTournament"];

/**
 * O que quem joga tem de fazer numa série, do prazo mais curto ao mais longo:
 * o veto corre em segundos, a resposta tem a janela de confirmação. "Resolver"
 * fica de fora: é da organização, que a vê nas contestações.
 */
const PLAYER_TASKS = [
  "veto",
  "ready",
  "respond",
  "report",
] as const satisfies Schemas["ViewerTask"][];

type PlayerTask = (typeof PLAYER_TASKS)[number];

function isPlayerTask(task: Schemas["ViewerTask"]): task is PlayerTask {
  return PLAYER_TASKS.some((playerTask) => playerTask === task);
}

/** Situações em que a participação de quem vê já acabou. */
const OVER: Schemas["Situation"][] = [
  "eliminated",
  "withdrawn",
  "missed_check_in",
  "champion",
];

/**
 * O que a próxima série espera de quem vê, se for dele a vez de agir. Com o
 * adversário ocupado em outra (round robin), ainda não é.
 */
export function playerTask(entry: Entry) {
  const task = entry.next_match?.task;
  if (entry.next_match?.opponent_playing) return undefined;
  return task && isPlayerTask(task) ? task : undefined;
}

/** Só anda se quem vê agir: o check-in que falta ou a vez dele na série. */
export function waitsForViewer(entry: Entry) {
  return (
    entry.situation === "check_in_pending" || playerTask(entry) !== undefined
  );
}

/** Do prazo mais curto ao mais longo; o check-in, que vai até o início, no fim. */
export function byUrgency(a: Entry, b: Entry) {
  const rank = (entry: Entry) => {
    const task = playerTask(entry);
    return task ? PLAYER_TASKS.indexOf(task) : PLAYER_TASKS.length;
  };
  return rank(a) - rank(b);
}

/**
 * Separa os torneios de quem vê em `active`, do que espera por ele ao que só
 * espera a data (dentro de cada grupo, a ordem da API), e `over`: encerrados
 * ou em que a participação dele acabou, mesmo com o torneio em andamento.
 */
export function splitMine(entries: Entry[]) {
  const rank = (entry: Entry) =>
    waitsForViewer(entry) ? 0 : entry.situation === "playing" ? 1 : 2;
  const isOver = (entry: Entry) =>
    entry.tournament.status === "finished" || OVER.includes(entry.situation);
  return {
    active: entries
      .filter((entry) => !isOver(entry))
      .toSorted((a, b) => rank(a) - rank(b)),
    over: entries.filter(isOver),
  };
}
