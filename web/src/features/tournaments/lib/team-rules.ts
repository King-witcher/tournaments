import type { components } from "@/shared/api/schema";

type TeamRules = components["schemas"]["TeamRules"];

/** O preset do formulário: a API só guarda as regras, e ele sai delas. */
export type TeamPreset = "captain" | "everyone" | "custom";

/** Os presets que têm valores (Personalizado é o resto). */
export const TEAM_PRESETS: Record<"captain" | "everyone", TeamRules> = {
  captain: {
    veto: "captain",
    ban: "captain",
    pick: "captain",
    report: "captain",
  },
  everyone: {
    veto: "anyone",
    ban: "anyone",
    pick: "each_player",
    report: "anyone",
  },
};

const same = (a: TeamRules, b: TeamRules) =>
  a.veto === b.veto &&
  a.ban === b.ban &&
  a.pick === b.pick &&
  a.report === b.report;

/** O preset que as regras formam; nenhum, Personalizado. */
export function teamPresetOf(rules: TeamRules): TeamPreset {
  if (same(rules, TEAM_PRESETS.captain)) return "captain";
  if (same(rules, TEAM_PRESETS.everyone)) return "everyone";
  return "custom";
}
