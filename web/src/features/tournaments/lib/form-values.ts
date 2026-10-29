import type { components } from "@/shared/api/schema";

type Schemas = components["schemas"];
type Locale = Schemas["Locale"];

export type NoRepeatChoice = "off" | Schemas["NoRepeatScope"];

/** Tamanhos de time do TDM (2v2, 3v3, 4v4). */
export const TEAM_SIZES = [2, 3, 4];

/** Campos do formulário de torneio (criação e edição), no idioma original. */
export type TournamentValues = {
  name: string;
  slug: string;
  locale: Locale;
  description: string;
  rules: string;
  visibility: Schemas["Visibility"];
  /** `null` com o campo vazio (o `required` do campo não deixa enviar assim). */
  maxParticipants: number | null;
  /** Duelo ou TDM. */
  mode: Schemas["TournamentMode"];
  /**
   * Jogadores por time no TDM. No duelo o valor fica guardado (voltar ao TDM
   * antes de salvar o devolve), mas vale 1: use `teamSizeOf`.
   */
  teamSize: number;
  /** O formato da fase única: eliminação, round robin ou suíço. */
  stageFormat: Schemas["StageFormat"]["format"];
  /** Na eliminação: simples ou dupla. */
  bracket: Schemas["Bracket"];
  grandFinalReset: boolean;
  bestOf: number;
  /** Na eliminação: o melhor de N da final (ou da grande final). */
  finalsBestOf: number;
  /** No suíço: as rodadas; `null` é o automático (⌈log2 N⌉). */
  swissRounds: number | null;
  bansPerGame: number;
  noRepeat: NoRepeatChoice;
  /** TDM: quem não repete, cada jogador ou o time inteiro (no duelo, o jogador). */
  noRepeatWho: Schemas["NoRepeatWho"];
  /** TDM: dois do mesmo time podem escolher o mesmo campeão no mesmo mapa. */
  teamDuplicates: boolean;
  /** Início em ISO (UTC); `null` sem data, só no rascunho. */
  startsAt: string | null;
  /** Janela de check-in em minutos; `null` sem check-in. */
  checkInMinutes: number | null;
  /** Janela de contestação de resultado, em minutos. */
  contestMinutes: number;
  /** Os mapas contam para o rating da comunidade (muda até começar). */
  countsForRating: boolean;
  /** Como sai o mapa de cada jogo. */
  mapMode: Schemas["MapMode"];
  /** Slugs do pool de mapas; `null` na criação: o padrão da API. */
  mapPool: string[] | null;
  /** Ban/pick de campeões a cada mapa. */
  championPhase: boolean;
  /** Segundos de cada passo do veto. */
  vetoSeconds: number;
  /** TDM: segundos da escolha simultânea de campeões. */
  teamPickSeconds: number;
  /** O envio do jogador exige screenshot do placar (o campo só aparece com screenshots na API). */
  screenshotsRequired: boolean;
};

/** Os textos de uma tradução. */
export type TranslationValues = {
  name: string;
  description: string;
  rules: string;
};

/**
 * Uma tradução no formulário. Desligada, não vai no envio (e, se existia, é
 * apagada ao salvar), mas o texto fica: ligar de novo antes de salvar o traz
 * de volta.
 */
export type TranslationDraft = TranslationValues & { enabled: boolean };

/** As traduções do formulário, por idioma diferente do original. */
export type Translations = Partial<Record<Locale, TranslationDraft>>;

/** A tradução que ainda não existe. */
export const EMPTY_TRANSLATION: TranslationDraft = {
  enabled: false,
  name: "",
  description: "",
  rules: "",
};

/**
 * Padrão do duelo: público, dupla sem reset, md3 e md5 na grande final, 16
 * vagas, 1 ban, check-in de 30 min, 10 min para contestar, veto de mapas no
 * pool padrão com ban/pick de campeões e 30 s por passo. Trocado para o TDM:
 * times de 2, a regra de não repetição por jogador, sem campeão igual no
 * time e 60 s para a escolha simultânea.
 */
export function defaultValues(locale: Locale): TournamentValues {
  return {
    name: "",
    slug: "",
    locale,
    description: "",
    rules: "",
    visibility: "public",
    maxParticipants: 16,
    mode: "duel",
    teamSize: 1,
    stageFormat: "elimination",
    bracket: "double",
    grandFinalReset: false,
    bestOf: 3,
    finalsBestOf: 5,
    swissRounds: null,
    bansPerGame: 1,
    noRepeat: "off",
    noRepeatWho: "player",
    teamDuplicates: false,
    startsAt: null,
    checkInMinutes: 30,
    contestMinutes: 10,
    countsForRating: true,
    mapMode: "veto",
    mapPool: null,
    championPhase: true,
    vetoSeconds: 30,
    teamPickSeconds: 60,
    screenshotsRequired: false,
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
    mode: tournament.mode,
    teamSize: tournament.team_size,
    stageFormat: stage.format,
    // Fora da eliminação, os campos dela ficam no padrão, para quem trocar
    // de formato no formulário.
    bracket: stage.format === "elimination" ? stage.bracket : "double",
    grandFinalReset:
      stage.format === "elimination" && (stage.grand_final_reset ?? false),
    bestOf: stage.best_of,
    finalsBestOf: stage.format === "elimination" ? stage.finals_best_of : 5,
    swissRounds: stage.format === "swiss" ? (stage.rounds ?? null) : null,
    bansPerGame: rules.bans_per_game,
    noRepeat: rules.no_repeat?.scope ?? "off",
    noRepeatWho: rules.no_repeat?.who ?? "player",
    teamDuplicates: rules.team_duplicates ?? false,
    startsAt: tournament.starts_at ?? null,
    checkInMinutes: tournament.check_in_minutes ?? null,
    contestMinutes: tournament.contest_minutes,
    countsForRating: tournament.counts_for_rating,
    mapMode: tournament.veto.map_mode,
    mapPool: tournament.veto.map_pool,
    championPhase: tournament.veto.champion_phase,
    vetoSeconds: tournament.veto.step_seconds,
    teamPickSeconds: tournament.veto.team_pick_seconds,
    screenshotsRequired: tournament.screenshots_required,
  };
}

/** É TDM: times, capitão e escolha simultânea de campeões. */
export const isTdm = (values: Pick<TournamentValues, "mode">) =>
  values.mode === "tdm";

/** Jogadores por time que valem: 1 no duelo. */
export const teamSizeOf = (
  values: Pick<TournamentValues, "mode" | "teamSize">,
) => (isTdm(values) ? values.teamSize : 1);

/** O pool do formulário: o escolhido, ou o padrão (`default_pool`) enquanto não mexeram. */
export function poolOf(values: TournamentValues, maps: Schemas["Map"][]) {
  return (
    values.mapPool ?? maps.filter((m) => m.default_pool).map((m) => m.slug)
  );
}

/** O maior melhor de N das séries: o pool precisa de um mapa para cada jogo. */
export const longestSeries = (values: TournamentValues) =>
  values.stageFormat === "elimination"
    ? Math.max(values.bestOf, values.finalsBestOf)
    : values.bestOf;

/** A tradução de `locale` que o torneio já tem (ligada), ou uma vazia. */
export function translationFrom(
  tournament: Schemas["Tournament"],
  locale: Locale,
): TranslationDraft {
  const saved = tournament.translations.find((x) => x.locale === locale);
  return saved
    ? {
        enabled: true,
        name: saved.name,
        description: saved.description,
        rules: saved.rules,
      }
    : EMPTY_TRANSLATION;
}

/** O que salvar a tradução de `locale` faz: gravar (PUT), apagar ou nada. */
export function translationChange(
  tournament: Schemas["Tournament"],
  locale: Locale,
  draft: TranslationDraft,
): "put" | "delete" | undefined {
  const saved = translationFrom(tournament, locale);
  if (!draft.enabled) return saved.enabled ? "delete" : undefined;
  const changed =
    !saved.enabled ||
    draft.name !== saved.name ||
    draft.description !== saved.description ||
    draft.rules !== saved.rules;
  return changed ? "put" : undefined;
}

/**
 * As escolhas de não repetição. "Na fase" só se distingue de "no torneio"
 * quando houver mais de uma fase: fica de fora, a não ser que o torneio já
 * a tenha.
 */
export function noRepeatChoices(current: NoRepeatChoice): NoRepeatChoice[] {
  return current === "stage"
    ? ["off", "match", "stage", "tournament"]
    : ["off", "match", "tournament"];
}

/**
 * Se a data de início não deixa tempo para inscrição e check-in: `past` (já
 * passou) ou `window` (a janela de check-in, contada a partir do início, já
 * abriu em `now`). `null` sem data ou com tempo de sobra.
 */
export function startTiming(
  startsAt: string | null,
  checkInMinutes: number | null,
  now: Date,
): "past" | "window" | null {
  if (startsAt === null) return null;
  const start = new Date(startsAt).getTime();
  if (start <= now.getTime()) return "past";
  if (
    checkInMinutes !== null &&
    start - checkInMinutes * 60_000 <= now.getTime()
  ) {
    return "window";
  }
  return null;
}

/** Corpo da criação: rascunho, ou já publicado com `publish`. */
export function createBody(
  values: TournamentValues,
  translations: Translations,
  publish: boolean,
) {
  return {
    ...textsBody(values),
    ...setupBody(values),
    ...scheduleBody(values),
    slug: values.slug,
    locale: values.locale,
    translations: Object.entries(translations).flatMap(([locale, draft]) =>
      draft?.enabled
        ? [
            {
              locale: locale as Locale,
              name: draft.name,
              description: draft.description,
              rules: draft.rules,
            },
          ]
        : [],
    ),
    publish,
  };
}

/**
 * Corpo do PATCH: o formulário inteiro (a API ignora o que não mudou). Com o
 * torneio já começado, formato, data e check-in não vão.
 */
export function updateBody(values: TournamentValues, setupLocked: boolean) {
  return {
    ...textsBody(values),
    ...(!setupLocked && { ...setupBody(values), ...scheduleBody(values) }),
  };
}

/** O que muda até o torneio encerrar: textos, visibilidade, janela de contestação e screenshot obrigatório. */
function textsBody(values: TournamentValues) {
  return {
    name: values.name,
    description: values.description,
    rules: values.rules,
    visibility: values.visibility,
    contest_minutes: values.contestMinutes,
    screenshots_required: values.screenshotsRequired,
  };
}

/** Data de início e janela de check-in no formato da API. */
function scheduleBody(values: TournamentValues) {
  return {
    starts_at: values.startsAt,
    check_in_minutes: values.checkInMinutes,
  };
}

/**
 * Modalidade, vagas, regra de campeões, fases, veto e "conta para o rating"
 * no formato da API. No duelo, o que é só do TDM vai com o valor do duelo
 * (time de 1, regra por jogador, sem campeão igual no time).
 */
function setupBody(values: TournamentValues) {
  const tdm = isTdm(values);
  return {
    mode: values.mode,
    team_size: teamSizeOf(values),
    counts_for_rating: values.countsForRating,
    veto: {
      map_mode: values.mapMode,
      // Sem mexer, na criação, vale o pool padrão da API.
      map_pool: values.mapPool ?? undefined,
      champion_phase: values.championPhase,
      step_seconds: values.vetoSeconds,
      team_pick_seconds: values.teamPickSeconds,
    },
    // Vazio não passa do `required` do campo; se passar, a API recusa e o
    // erro aparece no campo.
    max_participants: values.maxParticipants ?? 0,
    champion_rules: {
      bans_per_game: values.bansPerGame,
      no_repeat:
        values.noRepeat === "off"
          ? null
          : {
              who: tdm ? values.noRepeatWho : ("player" as const),
              scope: values.noRepeat,
            },
      team_duplicates: tdm && values.teamDuplicates,
    },
    stages: [stageBody(values)],
  };
}

/** A fase única no formato da API. */
function stageBody(values: TournamentValues): Schemas["StageFormat"] {
  switch (values.stageFormat) {
    case "round_robin":
      return { format: "round_robin", best_of: values.bestOf };
    case "swiss":
      return {
        format: "swiss",
        best_of: values.bestOf,
        rounds: values.swissRounds,
      };
    case "elimination":
      return {
        format: "elimination",
        bracket: values.bracket,
        grand_final_reset:
          values.bracket === "double" && values.grandFinalReset,
        best_of: values.bestOf,
        finals_best_of: values.finalsBestOf,
      };
  }
}
