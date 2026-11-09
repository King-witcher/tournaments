import type { TournamentValues, TranslationValues } from "./form-values";

/** Os campos de uma fase que um erro da API pode apontar. */
export type StageFieldName =
  | "format"
  | "bestOf"
  | "swissRounds"
  | "tiebreakers"
  | "advance";

/**
 * Um campo de uma fase: `stages.1.bestOf` é o melhor de N da segunda fase. É
 * também o `name` do controle dele.
 */
export type StageField = `stages.${number}.${StageFieldName}`;

/** O campo `name` da fase `index` (a partir de 0). */
export const stageField = (index: number, name: StageFieldName): StageField =>
  `stages.${index}.${name}`;

/** O campo é de uma fase. */
export const isStageField = (field: string): field is StageField =>
  field.startsWith("stages.");

/** A fase (a partir de 0) de um campo de fase. */
export const stageIndexOf = (field: StageField) => Number(field.split(".")[1]);

/** Campo do formulário de torneio que um erro da API pode apontar. */
export type TournamentField =
  | StageField
  | keyof Pick<
      TournamentValues,
      | "name"
      | "slug"
      | "description"
      | "rules"
      | "maxParticipants"
      | "teamSize"
      | "bansPerSeries"
      | "pickOrder"
      | "noRepeat"
      | "startsAt"
      | "contestMinutes"
      | "mapPool"
      | "vetoSeconds"
      | "teamPickSeconds"
    >;

/** Os erros de cada campo, já traduzidos. */
export type FieldErrors<F extends string = TournamentField> = Partial<
  Record<F, string>
>;

/**
 * O campo que cada código de erro da criação, do PATCH e da tradução aponta.
 * O resto (torneio já começou, encerrado, sem permissão, rede) é do
 * formulário inteiro. Faltar campeão depende de vagas, séries, bans e da
 * regra de não repetição (no TDM, também do tamanho do time, de quem não
 * repete e de campeão igual no time), mas na prática só acontece com a regra
 * ligada, e a dica dela já explica a conferência: o erro fica nela. As vagas
 * contam jogadores: o mínimo de dois times vem no erro das vagas.
 */
const FIELD_OF_CODE: Record<string, TournamentField> = {
  "tournament.name_invalid": "name",
  "tournament.slug_invalid": "slug",
  "tournament.slug_taken": "slug",
  "tournament.slug_reserved": "slug",
  "tournament.description_invalid": "description",
  "tournament.rules_invalid": "rules",
  "tournament.rules_image_invalid": "rules",
  "tournament.rules_images_too_many": "rules",
  "tournament.max_participants_invalid": "maxParticipants",
  "tournament.max_participants_below_registered": "maxParticipants",
  "tournament.team_size_invalid": "teamSize",
  "format.too_many_participants": "maxParticipants",
  "format.not_enough_players": "maxParticipants",
  "tournament.bans_invalid": "bansPerSeries",
  "champions.pool_exhausted": "noRepeat",
  "tournament.contest_minutes_invalid": "contestMinutes",
  "tournament.map_unknown": "mapPool",
  "tournament.map_pool_repeated": "mapPool",
  "tournament.map_pool_too_small": "mapPool",
  "tournament.map_pool_too_large": "mapPool",
  "tournament.veto_seconds_invalid": "vetoSeconds",
  "tournament.team_pick_seconds_invalid": "teamPickSeconds",
  "tournament.blind_needs_mirror": "pickOrder",
  "tournament.pick_order_invalid": "pickOrder",
};

/**
 * Os erros que apontam um campo da fase `params.stage` (a partir de 1; sem
 * ele, a primeira).
 */
const STAGE_FIELD_OF_CODE: Record<string, StageFieldName> = {
  "format.invalid_best_of": "bestOf",
  "format.invalid_rounds": "swissRounds",
  "format.invalid_tiebreakers": "tiebreakers",
  "stage.elimination_not_last": "format",
  "stage.invalid_advance": "advance",
  "stage.entry_rules_first": "advance",
};

/**
 * O campo do torneio que o erro aponta, ou `undefined` se é do formulário. Os
 * erros de formato levam a fase em `params.stage`.
 */
export function tournamentFieldOf(
  code: string,
  params?: Record<string, number | string>,
): TournamentField | undefined {
  const stageName = STAGE_FIELD_OF_CODE[code];
  if (stageName) {
    const stage = Number(params?.stage ?? 1);
    return stageField(
      Number.isInteger(stage) && stage > 0 ? stage - 1 : 0,
      stageName,
    );
  }
  return FIELD_OF_CODE[code];
}

/** Na tradução, só nome, descrição e regras têm erro próprio. */
export function translationFieldOf(
  code: string,
): keyof TranslationValues | undefined {
  const field = FIELD_OF_CODE[code];
  return field === "name" || field === "description" || field === "rules"
    ? field
    : undefined;
}

/**
 * O `name` do controle de um campo (da tradução de `locale`, se vier), para o
 * foco ir até ele quando a API o recusa.
 */
export function controlName(field: string, locale?: string) {
  return locale ? `translations.${locale}.${field}` : field;
}

/**
 * O seletor do controle de um campo: o `name` dele ou, num campo sem controle
 * nativo (a lista dos desempates), o `data-field` do bloco.
 */
export function controlSelector(field: string, locale?: string) {
  const name = controlName(field, locale);
  return `[name="${name}"], [data-field="${name}"]`;
}
