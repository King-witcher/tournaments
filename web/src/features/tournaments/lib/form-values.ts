import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];

export type NoRepeatChoice = "off" | Schemas["NoRepeatScope"];

/** Campos do formulário de torneio (criação e edição). */
export type TournamentValues = {
  name: string;
  slug: string;
  locale: Schemas["Locale"];
  description: string;
  rules: string;
  visibility: Schemas["Visibility"];
  maxParticipants: number;
  bracket: Schemas["Bracket"];
  grandFinalReset: boolean;
  bestOf: number;
  finalsBestOf: number;
  bansPerGame: number;
  noRepeat: NoRepeatChoice;
};

/** Padrão do duelo: público, dupla com reset, md3 e md5 na grande final, 16 vagas, 1 ban. */
export function defaultValues(locale: Schemas["Locale"]): TournamentValues {
  return {
    name: "",
    slug: "",
    locale,
    description: "",
    rules: "",
    visibility: "public",
    maxParticipants: 16,
    bracket: "double",
    grandFinalReset: true,
    bestOf: 3,
    finalsBestOf: 5,
    bansPerGame: 1,
    noRepeat: "off",
  };
}

/** Valores de um torneio existente, na fase única que a interface mostra. */
export function valuesFrom(
  tournament: Schemas["Tournament"],
): TournamentValues {
  const [stage] = tournament.stages;
  const rules = tournament.champion_rules;
  return {
    name: tournament.name,
    slug: tournament.slug,
    locale: tournament.locale,
    description: tournament.description,
    rules: tournament.rules,
    visibility: tournament.visibility,
    maxParticipants: tournament.max_participants,
    bracket: stage.bracket,
    grandFinalReset: stage.grand_final_reset ?? false,
    bestOf: stage.best_of,
    finalsBestOf: stage.finals_best_of,
    bansPerGame: rules.bans_per_game,
    noRepeat: rules.no_repeat?.scope ?? "off",
  };
}

/** Vagas, regra de campeões e fases no formato da API. */
export function setupBody(values: TournamentValues) {
  return {
    max_participants: values.maxParticipants,
    champion_rules: {
      bans_per_game: values.bansPerGame,
      no_repeat:
        values.noRepeat === "off"
          ? null
          : { who: "player" as const, scope: values.noRepeat },
    },
    stages: [
      {
        format: "elimination" as const,
        bracket: values.bracket,
        grand_final_reset:
          values.bracket === "double" && values.grandFinalReset,
        best_of: values.bestOf,
        finals_best_of: values.finalsBestOf,
      },
    ],
  };
}
