import type { components } from "@/shared/api/schema";
import { isStageField, type TournamentField } from "./form-errors";
import {
  isTdm,
  stepSecondsGroup,
  type TournamentValues,
  teamSizeOf,
} from "./form-values";
import { isFinished, setupEditable } from "./status";

type Status = components["schemas"]["TournamentStatus"];

/**
 * As seções das Configurações, uma página cada, na ordem do menu: Início e
 * inscrições logo depois de Geral, porque o início é o único campo sem padrão.
 */
export const SETTINGS_SECTIONS = [
  "general",
  "registration",
  "format",
  "maps",
  "champions",
  "results",
  "languages",
] as const;

export type SettingsSection = (typeof SETTINGS_SECTIONS)[number];

/** As seções que salvam campos do torneio (as traduções salvam à parte). */
export type ValuesSection = Exclude<SettingsSection, "languages">;

/**
 * Os campos que cada seção edita e manda ao salvar; o resto do corpo é o que
 * está salvo no torneio. Quem age pelo time fica no Formato, logo embaixo da
 * modalidade. O tempo de cada passo fica em Mapas ou em Campeões, conforme o
 * veto (`stepSecondsGroup`), e por isso está nas duas.
 */
export const SECTION_KEYS = {
  general: ["name", "slug", "description"],
  registration: ["startsAt", "admission", "checkInMinutes"],
  format: [
    "mode",
    "teamSize",
    "teamRules",
    "maxParticipants",
    "stages",
    "countsForRating",
  ],
  maps: ["mapMode", "vetoOrder", "mapPool", "vetoSeconds"],
  champions: [
    "championPhase",
    "bansPerSeries",
    "pickOrder",
    "mirror",
    "sameTeam",
    "noRepeat",
    "noRepeatWho",
    "vetoSeconds",
    "teamPickSeconds",
  ],
  results: ["contestMinutes", "screenshotsRequired"],
} as const satisfies Record<ValuesSection, readonly (keyof TournamentValues)[]>;

/** O `id` do título da seção: recebe o foco quando se chega pelo "Ir para". */
export const sectionTitleId = (section: SettingsSection) =>
  `settings-${section}`;

/** A seção da página da `[section]` do endereço, fora Geral (que é a raiz). */
export function sectionOfSegment(
  segment: string,
): Exclude<SettingsSection, "general"> | undefined {
  return SETTINGS_SECTIONS.find(
    (s): s is Exclude<SettingsSection, "general"> =>
      s === segment && s !== "general",
  );
}

/**
 * A seção que o `#` do endereço nomeia (`#maps`), se nomeia alguma: o
 * endereço de antes das páginas por seção.
 */
export function sectionOfHash(hash: string): SettingsSection | undefined {
  const name = hash.replace(/^#/, "");
  return SETTINGS_SECTIONS.find((s) => s === name);
}

/**
 * A seção em que se corrige um campo que a API recusou: a que o mostra com
 * os valores salvos. Um campo de tradução (`translation`) é de Outros idiomas.
 */
export function sectionOf(
  field: TournamentField,
  saved: TournamentValues,
  translation = false,
): SettingsSection {
  if (translation) return "languages";
  if (isStageField(field)) return "format";
  if (field === "vetoSeconds") {
    return stepSecondsGroup(saved) === "champions" ? "champions" : "maps";
  }
  const found = (Object.keys(SECTION_KEYS) as ValuesSection[]).find((s) =>
    (SECTION_KEYS[s] as readonly string[]).includes(field),
  );
  return found ?? "general";
}

/**
 * Por que a seção inteira está só para leitura no estado `status`, ou
 * `undefined` se ela ainda muda (inteira ou em parte). Geral sempre muda
 * (nome e descrição); Início e inscrições e Outros idiomas ainda mudam em
 * parte depois de começar (a inscrição; o nome e a descrição das traduções).
 */
export function sectionLock(
  section: SettingsSection,
  status: Status,
): "started" | "finished" | undefined {
  if (isFinished(status)) {
    return section === "general" || section === "languages"
      ? undefined
      : "finished";
  }
  if (setupEditable(status)) return undefined;
  return section === "format" || section === "maps" || section === "champions"
    ? "started"
    : undefined;
}

/**
 * O que vale de `key`: no duelo, o time é de 1 e as regras de time não
 * contam, mesmo que a tela guarde o que estava no TDM.
 */
function effective(values: TournamentValues, key: keyof TournamentValues) {
  if (key === "teamSize") return teamSizeOf(values);
  if (key === "teamRules" && !isTdm(values)) return null;
  return values[key];
}

/**
 * O valor de `key` mudou na tela em relação ao salvo. Listas e objetos (o
 * pool, os desempates, as regras de time) comparam pelo conteúdo.
 */
export function changed(
  saved: TournamentValues,
  values: TournamentValues,
  key: keyof TournamentValues,
) {
  const a = effective(saved, key);
  const b = effective(values, key);
  if (Object.is(a, b)) return false;
  return typeof a !== "object" || JSON.stringify(a) !== JSON.stringify(b);
}
