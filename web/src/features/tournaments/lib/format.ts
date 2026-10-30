import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];
type Stage = Schemas["StageFormat"];

/** Uma fase de eliminação (simples ou dupla). */
export type EliminationStage = Extract<Stage, { format: "elimination" }>;

/**
 * O formato como os textos o nomeiam e explicam: as duas eliminações (a
 * dupla com reset na grande final tem nome e explicação próprios), o round
 * robin e o suíço.
 */
export type FormatKind =
  | "single"
  | "double"
  | "doubleReset"
  | "round_robin"
  | "swiss";

/** A chave de eliminação como os textos a nomeiam e explicam. */
export function bracketKind(
  stage: Pick<EliminationStage, "bracket" | "grand_final_reset">,
) {
  return stage.bracket === "double" && stage.grand_final_reset
    ? "doubleReset"
    : stage.bracket;
}

/** O formato da fase, nos nomes dos textos. */
export function formatKind(stage: Stage): FormatKind {
  return stage.format === "elimination" ? bracketKind(stage) : stage.format;
}

/** Eliminação simples: o código das partidas não tem a parte da chave. */
export function isSingle(stage: Stage) {
  return stage.format === "elimination" && stage.bracket === "single";
}

/** Round robin ou suíço: a classificação sai de uma tabela de pontos. */
export function isTable(stage: Stage) {
  return stage.format !== "elimination";
}

/** O melhor de N da final, só na eliminação. */
export function finalsBestOf(stage: Stage) {
  return stage.format === "elimination" ? stage.finals_best_of : undefined;
}

/**
 * Rodadas do suíço com `entrants` no início: as configuradas (`rounds`) ou,
 * sem número, ⌈log2 N⌉; nunca mais que N − 1 (a mesma conta do `engine`).
 */
export function swissRounds(
  rounds: number | null | undefined,
  entrants: number,
) {
  if (entrants < 2) return 0;
  const auto = Math.ceil(Math.log2(entrants));
  return Math.min(rounds ?? auto, entrants - 1);
}

/** Até onde o campeão não se repete; `off` sem a regra. */
export function repeatScope(rules: Schemas["ChampionRules"]) {
  return rules.no_repeat?.scope ?? "off";
}

/**
 * A duração prevista de uma partida, em minutos (a mesma da API): o prazo
 * para confirmar ou contestar só começa a contar quando a partida completa
 * isso, para um envio adiantado não passar sem resposta.
 */
export const GAME_MINUTES = 15;

/** Mapas que vencem uma série melhor de N (N ímpar): 2 na md3, 3 na md5. */
export function mapsToWin(bestOf: number) {
  return Math.floor(bestOf / 2) + 1;
}

/**
 * O teto de participantes de cada formato no `engine` (round robin 32, suíço
 * 128), em lados da chave (no TDM, times); a eliminação vai até as vagas do
 * site.
 */
export const FORMAT_MAX_ENTRANTS: Partial<
  Record<Schemas["StageFormat"]["format"], number>
> = {
  round_robin: 32,
  swiss: 128,
};

/** Os limites de vagas da API, em jogadores (no TDM, o mínimo é de dois times). */
export const SLOTS = { min: 2, max: 1024 };

/**
 * As vagas que o formato aceita, em jogadores: de dois times até o teto do
 * formato vezes o tamanho do time (sem teto, o do site).
 */
export function slotLimits(
  format: Schemas["StageFormat"]["format"],
  teamSize: number,
) {
  const formatMax = FORMAT_MAX_ENTRANTS[format];
  return {
    min: Math.max(SLOTS.min, 2 * teamSize),
    max:
      formatMax === undefined
        ? SLOTS.max
        : Math.min(SLOTS.max, formatMax * teamSize),
    /** O teto é o do formato (o erro do campo é outro). */
    capped: formatMax !== undefined,
  };
}

/**
 * Times que as vagas comportam (a conta da API: as vagas divididas pelo
 * tamanho, sem fração) e os jogadores que sobram, de reserva.
 */
export function teamsIn(slots: number, teamSize: number) {
  return {
    teams: Math.floor(slots / teamSize),
    reserves: slots % teamSize,
  };
}
