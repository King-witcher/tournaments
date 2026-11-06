import type { components } from "@/shared/api/schema";
import type { TournamentField } from "./form-errors";
import { stepSecondsGroup, type TournamentValues } from "./form-values";
import { isFinished, setupEditable } from "./status";

type Status = components["schemas"]["TournamentStatus"];

/** As seções da aba Configurações, na ordem da página. */
export const SETTINGS_SECTIONS = [
  "general",
  "format",
  "teams",
  "maps",
  "champions",
  "registration",
  "results",
  "languages",
] as const;

export type SettingsSection = (typeof SETTINGS_SECTIONS)[number];

/** As seções que salvam campos do torneio (as traduções salvam à parte). */
export type ValuesSection = Exclude<SettingsSection, "languages">;

/**
 * Os campos que cada seção edita e manda ao salvar; o resto do corpo é o que
 * está salvo no torneio. O tempo de cada passo fica em Mapas ou em Campeões,
 * conforme o veto (`stepSecondsGroup`), e por isso está nas duas.
 */
export const SECTION_KEYS = {
  general: ["name", "slug", "description"],
  format: [
    "mode",
    "teamSize",
    "maxParticipants",
    "stageFormat",
    "bracket",
    "grandFinalReset",
    "bestOf",
    "finalsBestOf",
    "swissRounds",
    "tiebreakers",
    "countsForRating",
  ],
  teams: ["teamRules"],
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
  registration: ["startsAt", "admission", "checkInMinutes"],
  results: ["contestMinutes", "screenshotsRequired"],
} as const satisfies Record<ValuesSection, readonly (keyof TournamentValues)[]>;

/** O `id` do título de uma seção, alvo do foco do "Ir para". */
export const sectionHeadingId = (section: SettingsSection) =>
  `settings-${section}`;

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
 * (nome e descrição); Inscrições e prazos e Outros idiomas ainda mudam em
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
  return section === "format" ||
    section === "teams" ||
    section === "maps" ||
    section === "champions"
    ? "started"
    : undefined;
}
