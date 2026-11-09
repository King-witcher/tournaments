import type { components } from "@/shared/api/schema";

type GameMap = components["schemas"]["Map"];

/** O pool da RageCon, igual ao da QPL (temporada 4). */
const RAGECON = [
  "awoken",
  "blood-covenant",
  "corrupted-keep",
  "molten-falls",
  "ruins-of-sarnath",
  "vale-of-pnath",
  "deep-embrace",
];

const TOURNAMENT = [
  "awoken",
  "corrupted-keep",
  "molten-falls",
  "ruins-of-sarnath",
  "vale-of-pnath",
  "deep-embrace",
  "vestibule-of-exile",
];

const EXTENDED = [...TOURNAMENT, "insomnia"];

/** O pool do jogo na fila de duelo e de TDM. */
const RANKED = [...EXTENDED, "blood-covenant", "blood-run", "crucible"];

/** Os mapas que o modelo TDM deixa de fora; todo o resto do catálogo entra. */
const NOT_IN_TDM = ["dark-zone", "the-longest-yard"];

/** No TDM, os modelos de torneio têm o Blood Covenant no lugar do Deep Embrace. */
const forTdm = (slugs: string[]) =>
  slugs.map((slug) => (slug === "deep-embrace" ? "blood-covenant" : slug));

/**
 * Os modelos de pool de mapas, na ordem do `Select`, com os mapas por modalidade
 * (`null`: não aparece nela; `"catalog"`: o catálogo sem `NOT_IN_TDM`). A API
 * só guarda o pool; o modelo vive aqui e sai dele. Modelo novo: uma linha aqui
 * e o nome em `TournamentForm.veto.poolTemplates`, nos três idiomas.
 */
const TEMPLATES = [
  { id: "ragecon", duel: RAGECON, tdm: RAGECON },
  { id: "tournament", duel: TOURNAMENT, tdm: forTdm(TOURNAMENT) },
  { id: "tournament_extended", duel: EXTENDED, tdm: forTdm(EXTENDED) },
  { id: "ranked", duel: RANKED, tdm: RANKED },
  { id: "tdm", duel: null, tdm: "catalog" },
] as const;

export type MapPoolTemplateId = (typeof TEMPLATES)[number]["id"];

/** O que o `Select` mostra: um modelo ou Personalizado. */
export type MapPoolChoice = MapPoolTemplateId | "custom";

export type MapPoolTemplate = { id: MapPoolTemplateId; pool: string[] };

/** Os modelos da modalidade, com os mapas ativos de cada um, na ordem de `maps`; sem nenhum ativo, o modelo sai. */
export function mapPoolTemplates(
  maps: GameMap[],
  tdm: boolean,
): MapPoolTemplate[] {
  return TEMPLATES.flatMap(({ id, ...pools }) => {
    const slugs = tdm ? pools.tdm : pools.duel;
    if (slugs === null) return [];
    const pool = maps
      .filter((m) =>
        slugs === "catalog"
          ? !NOT_IN_TDM.includes(m.slug)
          : slugs.includes(m.slug),
      )
      .map((m) => m.slug);
    return pool.length > 0 ? [{ id, pool }] : [];
  });
}

const samePool = (a: string[], b: string[]) =>
  a.length === b.length && a.every((slug) => b.includes(slug));

/** O pool é o padrão de um torneio novo (`default_pool` de `GET /maps`). */
export function isDefaultPool(pool: string[], maps: GameMap[]): boolean {
  return samePool(
    pool,
    maps.filter((m) => m.default_pool).map((m) => m.slug),
  );
}

/**
 * O modelo que o pool forma (como conjunto); nenhum, Personalizado. Dois
 * modelos podem ter os mesmos mapas: vale `picked`, o que a pessoa acabou de
 * escolher, se for um deles; senão, o primeiro da lista.
 */
export function mapPoolChoiceOf(
  pool: string[],
  templates: MapPoolTemplate[],
  picked?: MapPoolTemplateId,
): MapPoolChoice {
  const matching = templates.filter((template) =>
    samePool(template.pool, pool),
  );
  return (
    matching.find((template) => template.id === picked)?.id ??
    matching[0]?.id ??
    "custom"
  );
}
