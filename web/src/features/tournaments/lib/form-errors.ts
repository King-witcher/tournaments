import type { TournamentValues, TranslationValues } from "./form-values";

/** Campo do formulário de torneio que um erro da API pode apontar. */
export type TournamentField = keyof Pick<
  TournamentValues,
  | "name"
  | "slug"
  | "description"
  | "rules"
  | "maxParticipants"
  | "teamSize"
  | "bestOf"
  | "swissRounds"
  | "bansPerSeries"
  | "noRepeat"
  | "startsAt"
  | "checkInMinutes"
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
  "tournament.max_participants_invalid": "maxParticipants",
  "tournament.max_participants_below_registered": "maxParticipants",
  "tournament.team_size_invalid": "teamSize",
  "format.invalid_best_of": "bestOf",
  "format.invalid_rounds": "swissRounds",
  "format.too_many_participants": "maxParticipants",
  "format.not_enough_players": "maxParticipants",
  "tournament.bans_invalid": "bansPerSeries",
  "champions.pool_exhausted": "noRepeat",
  "tournament.check_in_minutes_invalid": "checkInMinutes",
  "tournament.starts_at_past": "startsAt",
  "tournament.starts_at_too_soon": "startsAt",
  "tournament.contest_minutes_invalid": "contestMinutes",
  "tournament.map_unknown": "mapPool",
  "tournament.map_pool_repeated": "mapPool",
  "tournament.map_pool_too_small": "mapPool",
  "tournament.map_pool_too_large": "mapPool",
  "tournament.veto_seconds_invalid": "vetoSeconds",
  "tournament.team_pick_seconds_invalid": "teamPickSeconds",
};

/** O campo do torneio que o erro aponta, ou `undefined` se é do formulário. */
export function tournamentFieldOf(code: string): TournamentField | undefined {
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
