import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];
type Stage = Schemas["StageFormat"];

/** Um desempate de fase de tabela. */
export type Tiebreaker = Schemas["Tiebreaker"];

/** Os formatos de tabela: os que têm desempate. */
export type TableFormat = Extract<Stage["format"], "round_robin" | "swiss">;

/**
 * O padrão de cada formato, o mesmo do `engine` (`Tiebreakers::ROUND_ROBIN` e
 * `SWISS`). É também a ordem em que os desligados aparecem.
 */
export const DEFAULT_TIEBREAKERS: Record<TableFormat, Tiebreaker[]> = {
  round_robin: ["head_to_head", "map_diff", "frag_diff"],
  swiss: ["buchholz", "head_to_head", "map_diff", "frag_diff"],
};

export const isTableFormat = (format: Stage["format"]): format is TableFormat =>
  format !== "elimination";

/**
 * Os desempates de uma fase, na ordem; `null` na eliminação. A API sempre os
 * manda, mas o tipo os deixa opcionais porque a criação aceita sem eles (e
 * então vale o padrão).
 */
export function stageTiebreakers(stage: Stage): Tiebreaker[] | null {
  return stage.format === "elimination"
    ? null
    : (stage.tiebreakers ?? DEFAULT_TIEBREAKERS[stage.format]);
}

/**
 * O saldo de partidas desempata de verdade, menos no todos contra todos em
 * md1 e md2, em que ele só repete o saldo de séries (o dobro dele na md2).
 * Fora disso, aparece; gravado, continua gravado.
 */
export function counts(
  tiebreaker: Tiebreaker,
  format: TableFormat,
  bestOf: number,
) {
  return !(
    tiebreaker === "map_diff" &&
    format === "round_robin" &&
    (bestOf === 1 || bestOf === 2)
  );
}

/** Os desempates que contam, na ordem da fase: os da tela e do resumo. */
export function countingTiebreakers(
  list: Tiebreaker[],
  format: TableFormat,
  bestOf: number,
) {
  return list.filter((t) => counts(t, format, bestOf));
}

/** Os que o formato oferece, que contam e estão desligados, na ordem padrão. */
export function offTiebreakers(
  list: Tiebreaker[],
  format: TableFormat,
  bestOf: number,
) {
  return DEFAULT_TIEBREAKERS[format].filter(
    (t) => !list.includes(t) && counts(t, format, bestOf),
  );
}

/**
 * `list` com `tiebreaker` trocado de lugar com o vizinho que aparece na tela
 * (`step` −1 sobe, 1 desce), pulando o que não conta: ele fica onde estava.
 */
export function moveTiebreaker(
  list: Tiebreaker[],
  tiebreaker: Tiebreaker,
  step: -1 | 1,
  shown: (t: Tiebreaker) => boolean,
) {
  const from = list.indexOf(tiebreaker);
  let to = from + step;
  while (to >= 0 && to < list.length && !shown(list[to])) to += step;
  if (from < 0 || to < 0 || to >= list.length) return list;
  const next = [...list];
  [next[from], next[to]] = [next[to], next[from]];
  return next;
}
