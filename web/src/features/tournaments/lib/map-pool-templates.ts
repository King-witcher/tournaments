import type { components } from "@/shared/api/schema";

type GameMap = components["schemas"]["Map"];

/**
 * Os modelos de pool de mapas, na ordem do `Select`. A API só guarda o pool; o
 * modelo vive aqui e sai dele. Modelo novo: uma linha aqui e o nome em
 * `TournamentForm.veto.poolTemplates`, nos três idiomas.
 */
const TEMPLATES = [
  // O padrão vem dos mapas com `default_pool` de `GET /maps`.
  { id: "duel_default", slugs: "default_pool" },
  {
    id: "ragecon",
    slugs: [
      "awoken",
      "blood-covenant",
      "corrupted-keep",
      "molten-falls",
      "ruins-of-sarnath",
      "vale-of-pnath",
      "deep-embrace",
    ],
  },
] as const;

export type MapPoolTemplateId = (typeof TEMPLATES)[number]["id"];

/** O que o `Select` mostra: um modelo ou Personalizado. */
export type MapPoolChoice = MapPoolTemplateId | "custom";

export type MapPoolTemplate = { id: MapPoolTemplateId; pool: string[] };

const inTemplate = (
  slugs: "default_pool" | readonly string[],
  map: GameMap,
): boolean =>
  slugs === "default_pool" ? map.default_pool : slugs.includes(map.slug);

/** Os modelos com os mapas ativos de cada um, na ordem de `maps`; sem nenhum ativo, o modelo sai. */
export function mapPoolTemplates(maps: GameMap[]): MapPoolTemplate[] {
  return TEMPLATES.map(({ id, slugs }) => ({
    id,
    pool: maps.filter((m) => inTemplate(slugs, m)).map((m) => m.slug),
  })).filter((template) => template.pool.length > 0);
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
  const matching = templates.filter(
    (template) =>
      template.pool.length === pool.length &&
      template.pool.every((slug) => pool.includes(slug)),
  );
  return (
    matching.find((template) => template.id === picked)?.id ??
    matching[0]?.id ??
    "custom"
  );
}
