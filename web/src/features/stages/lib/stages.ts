import { isTable } from "@/features/tournaments/lib/format";
import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];
type Stage = Schemas["TournamentStage"];
type Tournament = Pick<Schemas["Tournament"], "status" | "stages">;

/** A fase já começou: tem chave e partidas. */
export function stageStarted(stage: Pick<Stage, "status">) {
  return stage.status === "in_progress" || stage.status === "finished";
}

/** As fases que começaram, na ordem. */
export function startedStages(tournament: Pick<Tournament, "stages">) {
  return tournament.stages.filter(stageStarted);
}

/** O torneio tem mais de uma fase (a interface só ganha o que for de fase assim). */
export function hasSeveralStages(tournament: Pick<Tournament, "stages">) {
  return tournament.stages.length > 1;
}

/** A fase atual: a última que começou ou, antes do início, a primeira. */
export function currentStage(tournament: Pick<Tournament, "stages">): Stage {
  return startedStages(tournament).at(-1) ?? tournament.stages[0];
}

/**
 * A fase seguinte à espera: o torneio segue em andamento, a anterior terminou
 * e ela não começou (sem check-in aberto ou com ele). `undefined` nos outros
 * casos.
 */
export function pendingStage(tournament: Tournament): Stage | undefined {
  if (tournament.status !== "in_progress") return undefined;
  return tournament.stages.find(
    (stage, index) =>
      index > 0 &&
      !stageStarted(stage) &&
      tournament.stages[index - 1].status === "finished",
  );
}

/** O valor de `?stage=` no endereço: uma fase, `final` (Classificação) ou nada. */
export type StageParam = number | "final" | undefined;

/** Lê `?stage=` do endereço; o que não for fase nem "final" vira `undefined`. */
export function parseStageParam(
  raw: string | string[] | undefined,
): StageParam {
  const value = Array.isArray(raw) ? raw[0] : raw;
  if (value === "final") return "final";
  if (value && /^\d+$/.test(value)) return Number(value);
  return undefined;
}

/**
 * A fase que a aba Chave mostra: a pedida, se já começou; senão, a atual
 * (`undefined`: a API escolhe a atual).
 */
export function bracketStage(
  tournament: Pick<Tournament, "stages">,
  requested: StageParam,
): number {
  const started = startedStages(tournament);
  const found = started.find((stage) => stage.number === requested);
  return (found ?? currentStage(tournament)).number;
}

/**
 * O que a aba Classificação mostra: a pedida, se já começou; senão, com o
 * torneio encerrado em mais de uma fase, a final; senão, a atual (ou, se ela
 * não tem tabela nem fim, a última com tabela).
 */
export function standingsStage(
  tournament: Tournament,
  requested: StageParam,
): number | "final" {
  const started = startedStages(tournament);
  if (requested === "final" && finalAvailable(tournament)) return "final";
  const found = started.find((stage) => stage.number === requested);
  if (found) return found.number;
  if (finalAvailable(tournament)) return "final";
  const current = currentStage(tournament);
  if (isTable(current)) return current.number;
  return (started.findLast((stage) => isTable(stage)) ?? current).number;
}

/**
 * A chave de uma fase com o estado dela no lugar do do torneio (o `status` da
 * API é o do torneio): a fase que terminou antes do fim do torneio sai como
 * encerrada, sem as dicas de quem ainda joga e com a classificação
 * definitiva.
 */
export function withStageStatus<
  B extends Pick<Schemas["TournamentBracket"], "status" | "stage">,
>(tournament: Pick<Tournament, "stages">, bracket: B): B {
  const stage = tournament.stages.find((s) => s.number === bracket.stage);
  return stage?.status === "finished" && bracket.status !== "finished"
    ? { ...bracket, status: "finished" }
    : bracket;
}

/** A fase é a última do torneio: quem fica em 1º nela venceu o torneio. */
export function isLastStage(
  tournament: Pick<Tournament, "stages">,
  stage: number,
) {
  return stage === tournament.stages.length;
}

/** A classificação final do torneio (a composta) só existe com ele encerrado e mais de uma fase. */
export function finalAvailable(tournament: Tournament) {
  return tournament.status === "finished" && tournament.stages.length > 1;
}
