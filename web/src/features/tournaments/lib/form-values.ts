import type { components } from "@/shared/api/schema";
import {
  isStageField,
  type StageFieldName,
  stageIndexOf,
  type TournamentField,
} from "./form-errors";
import { isEvenSeries } from "./format";
import {
  defaultStage,
  type StageValues,
  stageBody,
  stageValuesFrom,
} from "./stage-values";
import { setupEditable } from "./status";
import { TEAM_PRESETS } from "./team-rules";

type Schemas = components["schemas"];
type Locale = Schemas["Locale"];

export type NoRepeatChoice = "off" | Schemas["NoRepeatScope"];

/** Tamanhos de time do TDM (2v2, 3v3, 4v4). */
export const TEAM_SIZES = [2, 3, 4];

/** Os campos de um torneio na interface (configurações e cópia), no idioma original. */
export type TournamentValues = {
  name: string;
  slug: string;
  locale: Locale;
  description: string;
  rules: string;
  admission: Schemas["Admission"];
  /** `null` com o campo vazio (o `required` do campo não deixa enviar assim). */
  maxParticipants: number | null;
  /** Duelo ou TDM. */
  mode: Schemas["TournamentMode"];
  /**
   * Jogadores por time no TDM. No duelo o valor fica guardado (voltar ao TDM
   * antes de salvar o devolve), mas vale 1: use `teamSizeOf`.
   */
  teamSize: number;
  /** As fases, em sequência (de 1 a 4). */
  stages: StageValues[];
  /** Bans de campeão de cada lado na série. */
  bansPerSeries: number;
  /** Adversários podem usar o mesmo campeão na partida. */
  mirror: boolean;
  noRepeat: NoRepeatChoice;
  /** TDM: quem não repete, cada jogador ou o time inteiro (no duelo, o jogador). */
  noRepeatWho: Schemas["NoRepeatWho"];
  /** TDM: colegas de time podem usar o mesmo campeão na partida. */
  sameTeam: boolean;
  /** Início em ISO (UTC), só informativo; `null` sem data. */
  startsAt: string | null;
  /** Quem age pelo time: só vale nos times, e no duelo vai como está. */
  teamRules: Schemas["TeamRules"];
  /** O torneio tem check-in (a organização abre pelo botão). */
  checkIn: boolean;
  /** Janela de contestação de resultado, em minutos. */
  contestMinutes: number;
  /** Cada partida atualiza o rating dos jogadores (muda até começar). */
  countsForRating: boolean;
  /** Como sai o mapa de cada jogo. */
  mapMode: Schemas["MapMode"];
  /** Ordem do veto de mapas (só vale no modo veto): todos os bans primeiro, ou alternando. */
  vetoOrder: Schemas["VetoOrder"];
  /**
   * Slugs do pool de mapas; `null`: o padrão da API (a sala de veto sem mexer,
   * a cópia de um torneio sem nenhum mapa que ainda exista).
   */
  mapPool: string[] | null;
  /** Draft de campeões da série no site (`false`: fora do site). */
  championPhase: boolean;
  /** A ordem das escolhas de campeão, depois dos bans. */
  pickOrder: Schemas["PickOrder"];
  /** Segundos de cada passo do veto. */
  vetoSeconds: number;
  /** Segundos da janela simultânea de campeões (às cegas). */
  teamPickSeconds: number;
  /** O envio do jogador exige captura de tela do placar (o campo só aparece com capturas na API). */
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
 * O começo da sala de veto, que usa os campos de mapas e campeões do torneio:
 * veto de mapas alternado no pool padrão, 1 ban por série, espelho permitido,
 * draft de campeões no site e 30 s por passo, com as escolhas alternadas (às
 * cegas, 60 s para a janela simultânea). O torneio novo não parte daqui: sem
 * origem, a criação manda só nome, endereço e modalidade, e o resto é o padrão
 * da API (que também exige captura de tela quando há o bucket; a sala não tem
 * resultado, e o campo fica desligado).
 */
export function defaultValues(locale: Locale): TournamentValues {
  return {
    name: "",
    slug: "",
    locale,
    description: "",
    rules: "",
    admission: "open",
    maxParticipants: 16,
    mode: "duel",
    teamSize: 1,
    stages: [defaultStage()],
    bansPerSeries: 1,
    mirror: true,
    noRepeat: "off",
    noRepeatWho: "player",
    sameTeam: false,
    teamRules: TEAM_PRESETS.balanced,
    startsAt: null,
    checkIn: true,
    contestMinutes: 10,
    countsForRating: true,
    mapMode: "veto",
    vetoOrder: "alternating",
    mapPool: null,
    championPhase: true,
    pickOrder: "alternating",
    vetoSeconds: 30,
    teamPickSeconds: 60,
    screenshotsRequired: false,
  };
}

/** Valores de um torneio existente, com todas as fases. */
export function valuesFrom(
  tournament: Schemas["Tournament"],
): TournamentValues {
  const rules = tournament.champion_rules;
  return {
    name: tournament.name,
    slug: tournament.slug,
    locale: tournament.locale,
    description: tournament.description,
    rules: tournament.rules,
    admission: tournament.admission,
    maxParticipants: tournament.max_participants,
    mode: tournament.mode,
    teamSize: tournament.team_size,
    stages: tournament.stages.map(stageValuesFrom),
    bansPerSeries: rules.bans_per_series,
    mirror: rules.mirror,
    noRepeat: noRepeatFor(
      rules.no_repeat?.scope ?? "off",
      tournament.stages.length,
    ),
    noRepeatWho: rules.no_repeat?.who ?? "player",
    sameTeam: rules.same_team,
    teamRules: tournament.team_rules,
    startsAt: tournament.starts_at ?? null,
    checkIn: tournament.check_in,
    contestMinutes: tournament.contest_minutes,
    countsForRating: tournament.counts_for_rating,
    mapMode: tournament.veto.map_mode,
    vetoOrder: tournament.veto.order,
    mapPool: tournament.veto.map_pool,
    championPhase: tournament.veto.champion_phase,
    pickOrder: tournament.veto.pick_order,
    vetoSeconds: tournament.veto.step_seconds,
    teamPickSeconds: tournament.veto.team_pick_seconds,
    screenshotsRequired: tournament.screenshots_required,
  };
}

/**
 * O começo de um torneio novo copiado de `source`: a configuração e os textos
 * que se repetem a cada edição, no idioma original dela. Nome, endereço e
 * início ficam vazios, porque são da edição. O pool perde os mapas que saíram
 * do jogo (fora de `maps`); sem nenhum, fica o padrão.
 */
export function valuesFromSource(
  source: Schemas["Tournament"],
  maps: Schemas["Map"][],
): TournamentValues {
  const pool = source.veto.map_pool.filter((slug) =>
    maps.some((m) => m.slug === slug),
  );
  return {
    ...valuesFrom(source),
    name: "",
    slug: "",
    startsAt: null,
    mapPool: pool.length > 0 ? pool : null,
    // No duelo as regras não valem: virando TDM aqui, começa como todo time
    // novo (Balanceado, o padrão da API).
    teamRules:
      source.mode === "tdm" ? source.team_rules : TEAM_PRESETS.balanced,
    // A janela aberta só fica em quem já a tinha: não se escolhe mais.
    pickOrder:
      source.veto.pick_order === "open"
        ? "alternating"
        : source.veto.pick_order,
  };
}

/** O valor que a tela tem para o campo que um erro aponta (de uma fase ou do torneio). */
export function fieldValue(values: TournamentValues, field: TournamentField) {
  if (!isStageField(field)) return values[field];
  const stage = values.stages[stageIndexOf(field)];
  return stage?.[field.split(".")[2] as StageFieldName];
}

/** É TDM: times, capitão e escolha de campeões por jogador. */
export const isTdm = (values: Pick<TournamentValues, "mode">) =>
  values.mode === "tdm";

/** Jogadores por time que valem: 1 no duelo. */
export const teamSizeOf = (
  values: Pick<TournamentValues, "mode" | "teamSize">,
) => (isTdm(values) ? values.teamSize : 1);

/** O pool do formulário: o escolhido, ou o padrão (`default_pool`) enquanto não mexeram. */
export function poolOf(
  values: Pick<TournamentValues, "mapPool">,
  maps: Schemas["Map"][],
) {
  return (
    values.mapPool ?? maps.filter((m) => m.default_pool).map((m) => m.slug)
  );
}

/** As opções do tempo de cada passo do veto e do draft (a API aceita de 10 a 300). */
export const STEP_SECONDS = [15, 20, 30, 45, 60, 90, 120];

/** A janela da escolha simultânea (a API aceita de 15 a 300). */
export const TEAM_PICK_SECONDS = [15, 30, 45, 60, 90, 120, 180, 300];

/** O draft no site tem passos, com o tempo de cada passo: bans ou picks alternados. */
export function draftHasSteps(
  values: Pick<
    TournamentValues,
    "championPhase" | "pickOrder" | "bansPerSeries"
  >,
) {
  return (
    values.championPhase &&
    (values.pickOrder === "alternating" || values.bansPerSeries > 0)
  );
}

/** As escolhas de campeão são uma janela, com o tempo próprio dela. */
export const hasPickWindow = (
  values: Pick<TournamentValues, "championPhase" | "pickOrder">,
) => values.championPhase && values.pickOrder !== "alternating";

/**
 * As ordens das escolhas. A janela aberta só aparece para quem já a tem, o
 * torneio que começou antes de a ordem existir.
 */
export function pickOrderChoices(
  current: Schemas["PickOrder"],
): Schemas["PickOrder"][] {
  return current === "open"
    ? ["alternating", "blind", "open"]
    : ["alternating", "blind"];
}

/**
 * O grupo do formulário em que fica o tempo de cada passo: Mapas, com veto
 * ou votação (e então vale também para o draft); Campeões, quando só o draft
 * no site tem passos; nenhum, sem passo (o sorteio do mapa não tem).
 */
export function stepSecondsGroup(
  values: Pick<
    TournamentValues,
    "mapMode" | "championPhase" | "pickOrder" | "bansPerSeries"
  >,
) {
  if (values.mapMode === "veto" || values.mapMode === "vote") return "maps";
  return draftHasSteps(values) ? "champions" : null;
}

/** Os melhor de N que o torneio usa em todas as fases, sem repetir, do menor para o maior. */
export const seriesLengths = (values: Pick<TournamentValues, "stages">) =>
  [
    ...new Set(
      values.stages.flatMap((s) =>
        s.format === "elimination" ? [s.bestOf, s.finalsBestOf] : [s.bestOf],
      ),
    ),
  ].sort((a, b) => a - b);

/** O maior melhor de N das séries: o pool precisa de um mapa para cada jogo. */
export const longestSeries = (values: Pick<TournamentValues, "stages">) =>
  Math.max(...seriesLengths(values));

/** O melhor de N, se todas as séries do torneio têm o mesmo; senão, `undefined`. */
export function singleLength(values: Pick<TournamentValues, "stages">) {
  const lengths = seriesLengths(values);
  return lengths.length === 1 ? lengths[0] : undefined;
}

/**
 * Todas as séries são pares (só no todos contra todos): o veto de mapas só
 * tem bans, e não há ordem a escolher.
 */
export const onlyEvenSeries = (values: Pick<TournamentValues, "stages">) =>
  seriesLengths(values).every(isEvenSeries);

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

/**
 * As traduções de um torneio novo nos idiomas `locales`: as de `source`,
 * ligadas e com os textos, mas sem o nome, que é da edição. Sem origem, vazias.
 */
export function translationsFromSource(
  source: Schemas["Tournament"] | undefined,
  locales: Locale[],
): Translations {
  return Object.fromEntries(
    locales.map((l) => {
      const saved = source ? translationFrom(source, l) : EMPTY_TRANSLATION;
      return [l, saved.enabled ? { ...saved, name: "" } : saved];
    }),
  );
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
 * A regra de não repetição como vale com `stages` fases: o escopo "torneio"
 * e o "fase" são o mesmo com uma fase só, e com várias o campeão não se
 * carrega entre fases, então "torneio" vale como "fase". A tela mostra e
 * envia a que existe para a contagem de fases.
 */
export function noRepeatFor<C extends NoRepeatChoice>(
  current: C,
  stages: number,
): C | "stage" | "tournament" {
  if (stages > 1) return current === "tournament" ? "stage" : current;
  return current === "stage" ? "tournament" : current;
}

/**
 * As escolhas de não repetição: com uma fase, "no torneio"; com várias, "na
 * fase" (o campeão não se carrega de uma fase para a outra).
 */
export function noRepeatChoices(stages: number): NoRepeatChoice[] {
  return stages > 1
    ? ["off", "series", "stage"]
    : ["off", "series", "tournament"];
}

/**
 * Corpo da criação sem origem: o mínimo (nome, endereço, idioma e
 * modalidade), com os padrões da API, e o pool lembrado da comunidade, se
 * houver. Sempre rascunho.
 */
export function minimalCreateBody(
  values: Pick<
    TournamentValues,
    "name" | "slug" | "locale" | "mode" | "teamSize"
  >,
  pool: string[] | null,
) {
  return {
    slug: values.slug,
    locale: values.locale,
    name: values.name,
    mode: values.mode,
    team_size: teamSizeOf(values),
    ...(pool && { veto: { map_pool: pool } }),
  };
}

/** Corpo da criação a partir de um torneio copiado: a configuração inteira. Sempre rascunho. */
export function createBody(
  values: TournamentValues,
  translations: Translations,
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
  };
}

/**
 * Corpo do PATCH: o formulário inteiro (a API ignora o que não mudou). O
 * endereço só vai no rascunho; com o torneio já começado, formato, data e
 * check-in não vão; encerrado, só o nome e a descrição.
 */
export function updateBody(
  values: TournamentValues,
  status: Schemas["TournamentStatus"],
) {
  if (status === "finished") {
    return { name: values.name, description: values.description };
  }
  return {
    ...textsBody(values),
    ...(status === "draft" && { slug: values.slug }),
    ...(setupEditable(status) && {
      ...setupBody(values),
      ...scheduleBody(values),
    }),
    // Em andamento, só as fases que não começaram mudam: vão todas, e a API
    // recusa a que começou e mudou.
    ...(status === "in_progress" && { stages: values.stages.map(stageBody) }),
  };
}

/** O que muda até o torneio encerrar: textos, tipo de inscrição, janela de contestação e captura de tela obrigatória. */
function textsBody(values: TournamentValues) {
  return {
    name: values.name,
    description: values.description,
    rules: values.rules,
    admission: values.admission,
    contest_minutes: values.contestMinutes,
    screenshots_required: values.screenshotsRequired,
  };
}

/** Data de início e check-in no formato da API. */
function scheduleBody(values: TournamentValues) {
  return {
    starts_at: values.startsAt,
    check_in: values.checkIn,
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
    team_rules: values.teamRules,
    veto: {
      map_mode: values.mapMode,
      order: values.vetoOrder,
      // Nulo, vale o pool padrão da API.
      map_pool: values.mapPool ?? undefined,
      champion_phase: values.championPhase,
      pick_order: values.pickOrder,
      step_seconds: values.vetoSeconds,
      team_pick_seconds: values.teamPickSeconds,
    },
    // Vazio não passa do `required` do campo; se passar, a API recusa e o
    // erro aparece no campo.
    max_participants: values.maxParticipants ?? 0,
    champion_rules: {
      bans_per_series: values.bansPerSeries,
      mirror: values.mirror,
      same_team: tdm && values.sameTeam,
      no_repeat:
        values.noRepeat === "off"
          ? null
          : {
              who: tdm ? values.noRepeatWho : ("player" as const),
              scope: noRepeatFor(values.noRepeat, values.stages.length),
            },
    },
    stages: values.stages.map(stageBody),
  };
}
