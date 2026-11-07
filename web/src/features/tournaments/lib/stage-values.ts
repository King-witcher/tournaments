import type { components } from "@/shared/api/schema";
import { DEFAULT_TIEBREAKERS, stageTiebreakers } from "./tiebreakers";

type Schemas = components["schemas"];

/** O formato de uma fase: eliminação, todos contra todos, suíço ou disputa de posições. */
export type StageKind = Schemas["StageFormat"]["format"];

/** Uma fase como o torneio a lê: o formato e a situação. */
export type StageRead = Schemas["TournamentStage"];

/** O máximo de fases de um torneio (o `MAX_STAGES` da API). */
export const MAX_STAGES = 4;

/** Os campos de uma fase na interface. */
export type StageValues = {
  format: StageKind;
  /** Na eliminação: simples ou dupla. */
  bracket: Schemas["Bracket"];
  grandFinalReset: boolean;
  bestOf: number;
  /** Na eliminação: o melhor de N da final (ou da grande final). */
  finalsBestOf: number;
  /** No suíço: as rodadas; `null` é o automático (⌈log2 N⌉). */
  swissRounds: number | null;
  /**
   * Nas fases de tabela: os desempates ligados, na ordem. Na eliminação fica
   * o padrão do todos contra todos, que não vai no envio; trocar de formato
   * põe o padrão do novo.
   */
  tiebreakers: Schemas["Tiebreaker"][];
  /**
   * Nas fases de tabela: quem decide o empate que sobra depois de todos os
   * critérios, no fim da fase.
   */
  finalTie: Schemas["FinalTie"];
  /** Da fase 2 em diante: quantos da fase anterior entram; `null`, todos. */
  advance: number | null;
  /** Da fase 2 em diante: check-in antes da fase. */
  checkIn: boolean;
};

/** A primeira fase de um torneio com a configuração padrão da interface. */
export function defaultStage(): StageValues {
  return {
    format: "elimination",
    bracket: "double",
    grandFinalReset: false,
    bestOf: 3,
    finalsBestOf: 5,
    swissRounds: null,
    tiebreakers: DEFAULT_TIEBREAKERS.round_robin,
    finalTie: "seed",
    advance: null,
    checkIn: false,
  };
}

/** Uma fase acrescentada: a disputa de posições em melhor de 3, com todos entrando. */
export function addedStage(): StageValues {
  return { ...defaultStage(), format: "placement" };
}

/** Os valores de uma fase lida da API. */
export function stageValuesFrom(stage: Schemas["StageFormat"]): StageValues {
  const table = stage.format === "round_robin" || stage.format === "swiss";
  return {
    format: stage.format,
    // Fora da eliminação, os campos dela ficam no padrão, para quem trocar
    // de formato no formulário.
    bracket: stage.format === "elimination" ? stage.bracket : "double",
    grandFinalReset:
      stage.format === "elimination" && (stage.grand_final_reset ?? false),
    bestOf: stage.best_of,
    finalsBestOf: stage.format === "elimination" ? stage.finals_best_of : 5,
    swissRounds: stage.format === "swiss" ? (stage.rounds ?? null) : null,
    tiebreakers: stageTiebreakers(stage) ?? DEFAULT_TIEBREAKERS.round_robin,
    finalTie: table ? (stage.final_tie ?? "seed") : "seed",
    advance: stage.advance ?? null,
    checkIn: stage.check_in ?? false,
  };
}

/**
 * A fase no formato da API. O corte e o check-in só existem da fase 2 em
 * diante (`index` 1): a primeira vai sem eles, mesmo que a tela os guarde.
 */
export function stageBody(
  stage: StageValues,
  index: number,
): Schemas["StageFormat"] {
  const entry =
    index > 0 ? { advance: stage.advance, check_in: stage.checkIn } : {};
  switch (stage.format) {
    case "round_robin":
      return {
        format: "round_robin",
        best_of: stage.bestOf,
        tiebreakers: stage.tiebreakers,
        final_tie: stage.finalTie,
        ...entry,
      };
    case "swiss":
      return {
        format: "swiss",
        best_of: stage.bestOf,
        rounds: stage.swissRounds,
        tiebreakers: stage.tiebreakers,
        final_tie: stage.finalTie,
        ...entry,
      };
    case "placement":
      return { format: "placement", best_of: stage.bestOf, ...entry };
    case "elimination":
      return {
        format: "elimination",
        bracket: stage.bracket,
        grand_final_reset: stage.bracket === "double" && stage.grandFinalReset,
        best_of: stage.bestOf,
        finals_best_of: stage.finalsBestOf,
        ...entry,
      };
  }
}

/** A fase já começou: o formato dela não muda mais. */
export const stageStarted = (stage: Pick<StageRead, "status"> | undefined) =>
  stage?.status === "in_progress" || stage?.status === "finished";
